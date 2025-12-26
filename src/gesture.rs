use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub struct GestureConfig {
    pub edge_size: f64,
    pub require_edge_start: bool,
}

impl GestureConfig {
    pub fn new(edge_size: f64, require_edge_start: bool) -> Self {
        Self {
            edge_size,
            require_edge_start,
        }
    }
}

#[derive(Debug)]
pub struct GestureState {
    position: (f64, f64),
    active_touches: u32,
    swipe: Option<Swipe>,
    two_finger: Option<TwoFingerPress>,
    config: GestureConfig,
    screen_dimensions: (f64, f64),
}

impl GestureState {
    pub fn new(config: GestureConfig, screen_width: f64, screen_height: f64) -> Self {
        Self {
            position: (0.0, 0.0),
            active_touches: 0,
            swipe: None,
            two_finger: None,
            config,
            screen_dimensions: (screen_width, screen_height),
        }
    }

    fn is_on_screen_edge(&self, x: f64, y: f64) -> bool {
        let edge_size = self.config.edge_size;
        let screen_width = self.screen_dimensions.0;
        let screen_height = self.screen_dimensions.1;

        // Check if position is within edge_size pixels from any screen edge
        x <= edge_size
            || y <= edge_size
            || x >= (screen_width - edge_size)
            || y >= (screen_height - edge_size)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GestureType {
    SwipeLeft,
    SwipeUp,
    SwipeDown,
    SwipeRight,
}

#[derive(Debug, Clone, Copy)]
pub struct Gesture {
    pub gesture_type: GestureType,
    pub num_fingers: u32,
}

impl Gesture {
    pub fn new(gesture_type: GestureType, num_fingers: u32) -> Self {
        Self {
            gesture_type,
            num_fingers,
        }
    }
}

#[derive(Debug)]
struct Swipe {
    start_position: (f64, f64),
    start_time: Instant,
    min_swipe_distance: f64,
    max_swipe_duration: Duration,
}

impl Swipe {
    fn new(start_position: (f64, f64)) -> Self {
        tracing::debug!("New swipe with position {start_position:?}");
        Self {
            start_position,
            start_time: Instant::now(),
            min_swipe_distance: 100.0,
            max_swipe_duration: Duration::from_secs(1),
        }
    }
}

#[derive(Debug)]
struct TwoFingerPress {
    press_start_time: Instant,
    min_long_press_duration: Duration,
}

impl Default for TwoFingerPress {
    fn default() -> Self {
        Self {
            press_start_time: Instant::now(),
            min_long_press_duration: Duration::from_millis(800),
        }
    }
}

impl GestureState {
    pub fn update(&mut self, x: f64, y: f64) {
        tracing::trace!("Updating position to ({}, {})", x, y);
        self.position = (x, y);
    }

    pub fn handle_touch_down(&mut self, x: f64, y: f64) {
        self.position = (x, y);
        self.active_touches += 1;
        tracing::trace!("Active touches incremented {}", self.active_touches);
        tracing::debug!("Touch down at {:?}", self.position);

        match self.active_touches {
            1 => {
                // Check if swipe should be initiated based on edge detection
                if !self.config.require_edge_start || self.is_on_screen_edge(x, y) {
                    let swipe = Swipe::new(self.position);
                    tracing::debug!("New swipe {swipe:?}");
                    self.swipe = Some(swipe);
                } else {
                    tracing::debug!("Touch not on screen edge, swipe not initiated");
                }
            }
            2 => {
                self.two_finger = Some(TwoFingerPress::default());
            }
            _ => {}
        };
    }

    pub fn handle_touch_up(&mut self) -> Option<Gesture> {
        self.active_touches -= 1;
        if let Some(swipe) = self.swipe.take() {
            tracing::debug!("Swipe triggered {swipe:?}");
            let duration = swipe.start_time.elapsed();
            if duration > swipe.max_swipe_duration {
                return None;
            }

            let dx = self.position.0 - swipe.start_position.0;
            let dy = self.position.1 - swipe.start_position.1;
            let distance = ((dx * dx + dy * dy) as f64).sqrt();

            tracing::debug!("Swipe distance {distance}");
            if distance < swipe.min_swipe_distance {
                return None; // Too short to be a swipe
            }

            println!("{:?}", distance);
            // Determine swipe direction
            self.swipe = None;

            // Determine gesture type based on direction
            let gesture_type = if dx.abs() > dy.abs() {
                if dx > 0.0 {
                    GestureType::SwipeRight
                } else {
                    GestureType::SwipeLeft
                }
            } else {
                if dy > 0.0 {
                    GestureType::SwipeDown
                } else {
                    GestureType::SwipeUp
                }
            };

            // Create gesture with type and finger count
            return Some(Gesture::new(gesture_type, 1));
        }

        None
    }
}
