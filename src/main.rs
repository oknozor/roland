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
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

extern crate libc;
use libc::{O_RDONLY, O_RDWR, O_WRONLY};

use crate::gesture::GestureState;
use crate::output::get_output_dimensions;

mod config;
mod gesture;
mod output;

/// Touch Gesture Daemon
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, short)]
    config: PathBuf,

    /// Increase log verbosity (-v info, -vv debug, -vvv trace)
    #[arg(short, action = clap::ArgAction::Count)]
    verbose: u8,
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

    let log_level = match args.verbose {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let mut input = Libinput::new_with_udev(Interface);
    input.udev_assign_seat("seat0").unwrap();

    let (width, height) = get_output_dimensions().unwrap_or_else(|| {
        tracing::warn!("Could not detect output dimensions, defaulting to 1920x1080");
        (1920, 1080)
    });
    tracing::info!("Using screen dimensions: {}x{}", width, height);

    let config = config::GesturesConfig::from_path(&args.config).unwrap();
    let mut state = GestureState::new(config, width as f64, height as f64);

    loop {
        let poll_fd = PollFd::from_borrowed_fd(input.as_fd(), PollFlags::IN);
        poll(&mut [poll_fd], None).unwrap();

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


