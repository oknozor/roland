use clap::Parser;
use input::event::TouchEvent;
use input::event::touch::TouchEventPosition;
use input::{Event as InputEvent, Libinput, LibinputInterface};
use std::fs::{File, OpenOptions};
use std::os::unix::{fs::OpenOptionsExt, io::OwnedFd};
use std::path::Path;
use std::process::Command;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

extern crate libc;
use libc::{O_RDONLY, O_RDWR, O_WRONLY};

use crate::gesture::{GestureConfig, GestureState};

mod gesture;

/// Touch Gesture Daemon
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Size of screen edge zone in pixels
    #[arg(long, default_value_t = 50.0)]
    edge_size: f64,

    /// Require swipes to start from screen edges
    #[arg(long, default_value_t = true)]
    require_edge: bool,
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

    tracing::info!(
        "Starting Roland with edge_size={}px, require_edge_start={}",
        args.edge_size,
        args.require_edge
    );

    let config = GestureConfig::new(args.edge_size, args.require_edge);
    let mut state = GestureState::new(config, width as f64, height as f64);

    loop {
        input.dispatch().unwrap();
        for event in &mut input {
            match event {
                InputEvent::Touch(TouchEvent::Motion(touch_event)) => {
                    state.update(
                        touch_event.x_transformed(width),
                        touch_event.y_transformed(height),
                    );
                }
                InputEvent::Touch(TouchEvent::Down(touch_event)) => {
                    state.handle_touch_down(
                        touch_event.x_transformed(width),
                        touch_event.y_transformed(height),
                    );
                }
                InputEvent::Touch(TouchEvent::Up(_)) => {
                    if let Some(gesture) = state.handle_touch_up() {
                        tracing::info!("{gesture:?}");
                    }
                }
                _ => {}
            }
        }
    }
}

fn get_output_dimensions() -> Option<(u32, u32)> {
    #[derive(serde::Deserialize)]
    struct OutputData {
        logical: LogicalDimensions,
    }

    #[derive(serde::Deserialize)]
    struct LogicalDimensions {
        width: u32,
        height: u32,
    }

    let output = Command::new("niri")
        .arg("msg")
        .arg("-j")
        .arg("outputs")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let json_str = String::from_utf8(output.stdout).ok()?;
    let outputs: std::collections::HashMap<String, OutputData> =
        serde_json::from_str(&json_str).ok()?;

    if let Some(output_data) = outputs.values().next() {
        Some((output_data.logical.width, output_data.logical.height))
    } else {
        None
    }
}
