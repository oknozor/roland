use clap::Parser;
use input::event::TouchEvent;
use input::event::touch::{TouchEventPosition, TouchEventSlot};
use input::{Event as InputEvent, Libinput, LibinputInterface};
use rustix::event::{PollFd, PollFlags, poll};
use std::fs::{File, OpenOptions};
use std::os::unix::{
    fs::OpenOptionsExt,
    io::{AsFd, OwnedFd},
};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

extern crate libc;
use libc::{O_RDONLY, O_RDWR, O_WRONLY};

use crate::gesture::GestureState;

mod config;
mod gesture;

/// Touch Gesture Daemon
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Size of screen edge zone in pixels
    #[arg(long, short)]
    config: PathBuf,
}

struct Interface;

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(|file| file.into())
            .map_err(|err| err.raw_os_error().unwrap())
    }

    fn close_restricted(&mut self, fd: OwnedFd) {
        let _ = File::from(fd);
    }
}

fn main() {
    let args = Args::parse();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    let mut input = Libinput::new_with_udev(Interface);
    input.udev_assign_seat("seat0").unwrap();
    let (width, height) = get_output_dimensions().unwrap();

    let config = config::GesturesConfig::from_path(&args.config).unwrap();
    let mut state = GestureState::new(config, width as f64, height as f64);

    loop {
        // Wait for events using poll() to avoid busy-waiting
        let poll_fd = PollFd::from_borrowed_fd(input.as_fd(), PollFlags::IN);
        poll(&mut [poll_fd], -1).unwrap();

        // Process events when available
        input.dispatch().unwrap();
        for event in &mut input {
            match event {
                InputEvent::Touch(TouchEvent::Motion(touch_event)) => {
                    state.update(
                        touch_event.slot(),
                        touch_event.x_transformed(width),
                        touch_event.y_transformed(height),
                    );
                }
                InputEvent::Touch(TouchEvent::Down(touch_event)) => {
                    state.touch_down(
                        touch_event.slot(),
                        touch_event.x_transformed(width),
                        touch_event.y_transformed(height),
                    );
                }
                InputEvent::Touch(TouchEvent::Up(touch_event)) => {
                    state.touch_up(touch_event.slot());
                }
                _ => {}
            }
        }
    }
}

enum Compositor {
    Niri,
    Hyprland,
    Sway,
}

fn detect_compositor() -> Option<Compositor> {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_lowercase();

    match (
        desktop.as_str(),
        std::env::var("NIRI_SOCKET").is_ok(),
        std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok(),
        std::env::var("SWAYSOCK").is_ok(),
    ) {
        (_, true, _, _) => Some(Compositor::Niri),
        (_, _, true, _) => Some(Compositor::Hyprland),
        (_, _, _, true) => Some(Compositor::Sway),
        (d, _, _, _) if d.contains("niri") => Some(Compositor::Niri),
        (d, _, _, _) if d.contains("hyprland") => Some(Compositor::Hyprland),
        (d, _, _, _) if d.contains("sway") => Some(Compositor::Sway),
        _ => None,
    }
}

fn get_output_dimensions() -> Option<(u32, u32)> {
    match detect_compositor()? {
        Compositor::Niri => get_output_dimensions_niri(),
        Compositor::Hyprland => get_output_dimensions_hyprland(),
        Compositor::Sway => get_output_dimensions_sway(),
    }
}

fn get_output_dimensions_niri() -> Option<(u32, u32)> {
    #[derive(serde::Deserialize)]
    struct OutputData {
        logical: Option<LogicalDimensions>,
    }

    #[derive(serde::Deserialize)]
    struct LogicalDimensions {
        width: u32,
        height: u32,
    }

    let output = Command::new("niri")
        .args(["msg", "-j", "outputs"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let outputs: std::collections::HashMap<String, OutputData> =
        serde_json::from_slice(&output.stdout).ok()?;

    outputs
        .values()
        .find_map(|o| o.logical.as_ref().map(|l| (l.width, l.height)))
}

fn get_output_dimensions_hyprland() -> Option<(u32, u32)> {
    #[derive(serde::Deserialize)]
    struct Monitor {
        width: u32,
        height: u32,
        focused: bool,
    }

    let output = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let monitors: Vec<Monitor> = serde_json::from_slice(&output.stdout).ok()?;

    // prefer focused monitor, fall back to first
    monitors
        .iter()
        .find(|m| m.focused)
        .or_else(|| monitors.first())
        .map(|m| (m.width, m.height))
}

fn get_output_dimensions_sway() -> Option<(u32, u32)> {
    #[derive(serde::Deserialize)]
    struct Output {
        rect: Rect,
        focused: bool,
    }

    #[derive(serde::Deserialize)]
    struct Rect {
        width: u32,
        height: u32,
    }

    let output = Command::new("swaymsg")
        .args(["-t", "get_outputs", "--raw"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let outputs: Vec<Output> = serde_json::from_slice(&output.stdout).ok()?;

    outputs
        .iter()
        .find(|o| o.focused)
        .or_else(|| outputs.first())
        .map(|o| (o.rect.width, o.rect.height))
}
