//! Compositor-agnostic screen dimension discovery via Wayland protocols.
//!
//! Strategy:
//!   1. Try xdg-output-unstable-v1 for logical dimensions (handles HiDPI correctly).
//!   2. Fall back to wl_output physical size if xdg-output is unavailable.

use std::sync::{Arc, Mutex};

use wayland_client::{
    Connection, Dispatch, EventQueue, QueueHandle,
    globals::{GlobalList, GlobalListContents, registry_queue_init},
    protocol::{
        wl_output::{self, WlOutput},
        wl_registry::{self, WlRegistry},
    },
};
use wayland_protocols::xdg::xdg_output::zv1::client::{
    zxdg_output_manager_v1::ZxdgOutputManagerV1,
    zxdg_output_v1::{self, ZxdgOutputV1},
};

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Default)]
struct OutputState {
    // Physical size from wl_output (fallback)
    phys_width: i32,
    phys_height: i32,
    // Logical size from xdg-output (preferred)
    logical_width: Option<i32>,
    logical_height: Option<i32>,
    done: bool,
}

struct AppData {
    outputs: Vec<(WlOutput, Arc<Mutex<OutputState>>)>,
    xdg_manager: Option<ZxdgOutputManagerV1>,
    xdg_outputs: Vec<(ZxdgOutputV1, Arc<Mutex<OutputState>>)>,
}

// ── wl_registry ───────────────────────────────────────────────────────────────

impl Dispatch<WlRegistry, GlobalListContents> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Handled via GlobalList; nothing needed here.
    }
}

// ── wl_output ────────────────────────────────────────────────────────────────

impl Dispatch<WlOutput, Arc<Mutex<OutputState>>> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &WlOutput,
        event: wl_output::Event,
        data: &Arc<Mutex<OutputState>>,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        let mut s = data.lock().unwrap();
        match event {
            wl_output::Event::Mode { width, height, .. } => {
                s.phys_width = width;
                s.phys_height = height;
            }
            wl_output::Event::Done => {
                s.done = true;
            }
            _ => {}
        }
    }
}

// ── xdg-output ───────────────────────────────────────────────────────────────

impl Dispatch<ZxdgOutputManagerV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &ZxdgOutputManagerV1,
        _: wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZxdgOutputV1, Arc<Mutex<OutputState>>> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &ZxdgOutputV1,
        event: zxdg_output_v1::Event,
        data: &Arc<Mutex<OutputState>>,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        let mut s = data.lock().unwrap();
        match event {
            zxdg_output_v1::Event::LogicalSize { width, height } => {
                s.logical_width = Some(width);
                s.logical_height = Some(height);
            }
            zxdg_output_v1::Event::Done => {
                s.done = true;
            }
            _ => {}
        }
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Returns the logical (post-scale) dimensions of the first available output.
///
/// Uses xdg-output-unstable-v1 when the compositor supports it; falls back to
/// the physical mode size reported by wl_output.  Returns `None` if no output
/// is found or the Wayland connection fails.
pub fn get_output_dimensions() -> Option<(u32, u32)> {
    let conn = Connection::connect_to_env().ok()?;
    let (globals, mut queue): (GlobalList, EventQueue<AppData>) =
        registry_queue_init(&conn).ok()?;

    let qh = queue.handle();

    // Bind wl_output (version 2 minimum for Done event)
    let output_states: Vec<Arc<Mutex<OutputState>>> = globals
        .contents()
        .clone_list()
        .iter()
        .filter(|g| g.interface == "wl_output")
        .map(|g| {
            let state = Arc::new(Mutex::new(OutputState::default()));
            globals.registry().bind::<WlOutput, _, AppData>(
                g.name,
                2.min(g.version),
                &qh,
                state.clone(),
            );
            state
        })
        .collect();

    if output_states.is_empty() {
        tracing::warn!("No wl_output globals found");
        return None;
    }

    // Try to bind xdg-output manager
    let xdg_manager = globals
        .bind::<ZxdgOutputManagerV1, _, _>(&qh, 1..=3, ())
        .ok();

    let mut app = AppData {
        outputs: Vec::new(),
        xdg_manager,
        xdg_outputs: Vec::new(),
    };

    // If xdg-output is available, create an xdg output for each wl_output
    if let Some(ref mgr) = app.xdg_manager {
        for (wl_out, state) in app.outputs.iter() {
            let xdg_out = mgr.get_xdg_output(wl_out, &qh, state.clone());
            app.xdg_outputs.push((xdg_out, state.clone()));
        }
    }

    // Round-trip until all outputs report Done
    for _ in 0..5 {
        queue.roundtrip(&mut app).ok()?;
        if output_states.iter().all(|s| s.lock().unwrap().done) {
            break;
        }
    }

    // Pick first output with usable dimensions
    for state in &output_states {
        let s = state.lock().unwrap();
        if let (Some(w), Some(h)) = (s.logical_width, s.logical_height) {
            if w > 0 && h > 0 {
                tracing::debug!("Output logical dimensions (xdg-output): {}x{}", w, h);
                return Some((w as u32, h as u32));
            }
        }
        if s.phys_width > 0 && s.phys_height > 0 {
            tracing::debug!(
                "Output dimensions (wl_output fallback): {}x{}",
                s.phys_width,
                s.phys_height
            );
            return Some((s.phys_width as u32, s.phys_height as u32));
        }
    }

    tracing::warn!("Could not determine output dimensions from Wayland");
    None
}
