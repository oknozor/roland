use evdev::{AbsoluteAxisCode, AbsoluteAxisEvent, Device, EventSummary, KeyCode};
use std::fs::File;
use std::os::fd::OwnedFd;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct DeviceWrapper(Device);

/// Struct to track swipe gesture state
struct SwipeDetector {
    is_tracking: bool,
    start_position: (i32, i32),
    current_position: (i32, i32),
    start_time: Instant,
    min_swipe_distance: i32,
    max_swipe_duration: Duration,
}

impl SwipeDetector {
    fn new() -> Self {
        SwipeDetector {
            is_tracking: false,
            start_position: (0, 0),
            current_position: (0, 0),
            start_time: Instant::now(),
            min_swipe_distance: 100, // Minimum distance to consider as swipe
            max_swipe_duration: Duration::from_millis(500), // Max time for swipe
        }
    }

    fn start_tracking(&mut self, x: i32, y: i32) {
        self.is_tracking = true;
        self.start_position = (x, y);
        self.current_position = (x, y);
        self.start_time = Instant::now();
    }

    fn update_position(&mut self, x: i32, y: i32) {
        if self.is_tracking {
            self.current_position = (x, y);
        }
    }

    fn end_tracking(&mut self) -> Option<String> {
        if !self.is_tracking {
            return None;
        }

        self.is_tracking = false;
        let duration = self.start_time.elapsed();

        if duration > self.max_swipe_duration {
            return None; // Too slow to be a swipe
        }

        let dx = self.current_position.0 - self.start_position.0;
        let dy = self.current_position.1 - self.start_position.1;
        let distance = ((dx * dx + dy * dy) as f32).sqrt();

        if distance < self.min_swipe_distance as f32 {
            return None; // Too short to be a swipe
        }

        // Determine swipe direction
        if dx.abs() > dy.abs() {
            if dx > 0 {
                Some("swipe-right".to_string())
            } else {
                Some("swipe-left".to_string())
            }
        } else {
            if dy > 0 {
                Some("swipe-down".to_string())
            } else {
                Some("swipe-up".to_string())
            }
        }
    }
}

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
        let mut swipe_detector = SwipeDetector::new();
        let mut x_position: Option<i32> = None;
        let mut y_position: Option<i32> = None;

        loop {
            for event in device.fetch_events().unwrap() {
                match event.destructure() {
                    EventSummary::AbsoluteAxis(_axis_type, axis, value) => {
                        match axis {
                            AbsoluteAxisCode::ABS_X => {
                                x_position = Some(value);
                                println!("X axis moved to {}", value);
                            }
                            AbsoluteAxisCode::ABS_Y => {
                                y_position = Some(value);
                                println!("Y axis moved to {}", value);
                            }
                            AbsoluteAxisCode::ABS_MT_POSITION_X => {
                                x_position = Some(value);
                                println!("Multi-touch X position: {}", value);
                            }
                            AbsoluteAxisCode::ABS_MT_POSITION_Y => {
                                y_position = Some(value);
                                println!("Multi-touch Y position: {}", value);
                            }
                            _ => {
                                println!("The Axis {:?} was moved to {}", axis, value);
                            }
                        }

                        // Update swipe detector with current position
                        if let (Some(x), Some(y)) = (x_position, y_position) {
                            swipe_detector.update_position(x, y);
                        }
                    }
                    EventSummary::Key(_, KeyCode::BTN_TOUCH, 1) if !swipe_detector.is_tracking => {
                        // Touch started - begin swipe tracking
                        if let (Some(x), Some(y)) = (x_position, y_position) {
                            swipe_detector.start_tracking(x, y);
                            println!("Touch started at ({}, {})", x, y);
                        }
                    }
                    EventSummary::Key(_, KeyCode::BTN_TOUCH, 0) => {
                        // Touch ended - check for swipe
                        if let Some(swipe_direction) = swipe_detector.end_tracking() {
                            println!("Swipe detected: {}", swipe_direction);

                            // Handle different swipe directions
                            match swipe_direction.as_str() {
                                "swipe-left" => {
                                    println!("Handling swipe left gesture");
                                    // Add your swipe left logic here
                                }
                                "swipe-right" => {
                                    println!("Handling swipe right gesture");
                                    // Add your swipe right logic here
                                }
                                "swipe-up" => {
                                    println!("Handling swipe up gesture");
                                    // Add your swipe up logic here
                                }
                                "swipe-down" => {
                                    println!("Handling swipe down gesture");
                                    // Add your swipe down logic here
                                }
                                _ => {}
                            }
                        } else {
                            println!("Touch ended - no swipe detected");
                        }
                    }
                    e => println!("got event {:?}", e),
                }
            }
        }
    }
}
