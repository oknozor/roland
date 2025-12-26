use evdev::{Device, EventSummary, KeyCode};
use std::fs::File;
use std::os::fd::OwnedFd;
use std::path::PathBuf;

pub struct DeviceWrapper(Device);

impl DeviceWrapper {
    pub fn try_open(device_path: PathBuf) -> color_eyre::Result<Self> {
        let device_path_str = device_path.to_str().unwrap();
        let device_found = evdev::enumerate().any(|(path, _dev)| path == device_path);
        if !device_found {
            println!("Device {} not found", device_path_str);
            println!("Available devices: ");
            for (path, _) in evdev::enumerate() {
                println!("{}", path.to_str().unwrap());
            }
            std::process::exit(1);
        }

        let f = File::open(device_path_str)?;
        let fd = OwnedFd::from(f);
        let device = evdev::Device::from_fd(fd)?;
        Ok(DeviceWrapper(device))
    }

    pub fn listen(self) {
        let mut device = self.0;

        loop {
            for event in device.fetch_events().unwrap() {
                match event.destructure() {
                    EventSummary::Key(ev, KeyCode::KEY_A, 1) => {
                        println!("Key 'a' was pressed, got event: {:?}", ev);
                    }
                    EventSummary::Key(_, key_type, 0) => {
                        println!("Key {:?} was released", key_type);
                    }
                    EventSummary::AbsoluteAxis(_, axis, value) => {
                        println!("The Axis {:?} was moved to {}", axis, value);
                    }
                    e => println!("got event {:?}", e),
                }
            }
        }
    }
}
