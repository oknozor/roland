use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub struct GestureState {
    position: (f64, f64),
    active_touches: u32,
    swipe: Option<Swipe>,
    two_finger: Option<TwoFingerPress>,
}

#[derive(Debug)]
pub enum Gesture {
    SwipeLeft,
    SwipeUp,
    SwipeDown,
    SwipeRight,
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
        self.active_touches += 1;
        tracing::trace!("Active touches incremented {}", self.active_touches);

        match self.active_touches {
            1 => {
                let swipe = Swipe::new((x, y));
                tracing::debug!("New swipe {swipe:?}");
                self.swipe = Some(swipe);
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
            return if dx.abs() > dy.abs() {
                if dx > 0.0 {
                    Some(Gesture::SwipeRight)
                } else {
                    Some(Gesture::SwipeLeft)
                }
            } else {
                if dy > 0.0 {
                    Some(Gesture::SwipeDown)
                } else {
                    Some(Gesture::SwipeUp)
                }
            };
        }

        None
    }
}
