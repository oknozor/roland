use std::time::Instant;

use crate::config::GesturesConfig;

#[derive(Debug)]
pub struct GestureState {
    position: (f64, f64),
    active_touches: u32,
    swipe: Option<Gesture>,
    config: GesturesConfig,
    screen_dimensions: (f64, f64),
}

impl GestureState {
    pub fn new(config: GesturesConfig, screen_width: f64, screen_height: f64) -> Self {
        Self {
            position: (0.0, 0.0),
            active_touches: 0,
            swipe: None,
            config,
            screen_dimensions: (screen_width, screen_height),
        }
    }
}

#[derive(Debug)]
struct Gesture {
    start_position: (f64, f64),
    start_time: Instant,
    triggered: bool,
}

impl Gesture {
    fn new(start_position: (f64, f64)) -> Self {
        tracing::debug!("New gesture with position {start_position:?}");
        Self {
            start_position,
            start_time: Instant::now(),
            triggered: false,
        }
    }
}

impl GestureState {
    pub fn update(&mut self, x: f64, y: f64) {
        tracing::trace!("Updating position to ({}, {})", x, y);
        self.position = (x, y);
        self.check_press_gestures();
    }

    fn check_press_gestures(&mut self) {
        if let Some(gesture) = &mut self.swipe {
            if gesture.triggered {
                return;
            }

            let duration = gesture.start_time.elapsed();

            for gesture_config in self.config.gestures.iter() {
                if gesture_config.should_trigger(
                    self.active_touches,
                    gesture.start_position,
                    self.position,
                    self.screen_dimensions,
                    duration,
                ) {
                    tracing::info!("Triggering press gesture: {:?}", gesture_config);
                    gesture_config.run();
                    gesture.triggered = true;
                    break;
                }
            }
        }
    }

    pub fn handle_touch_down(&mut self, x: f64, y: f64) {
        self.position = (x, y);
        self.active_touches += 1;
        tracing::trace!("Active touches incremented {}", self.active_touches);
        tracing::debug!("Touch down at {:?}", self.position);
        self.swipe = Some(Gesture::new(self.position));
    }

    pub fn handle_touch_up(&mut self) {
        if let Some(swipe) = self.swipe.take() {
            if !swipe.triggered {
                for gesture in self.config.gestures.iter() {
                    let duration = swipe.start_time.elapsed();
                    if gesture.should_trigger(
                        self.active_touches,
                        swipe.start_position,
                        self.position,
                        self.screen_dimensions,
                        duration,
                    ) {
                        tracing::info!("Triggering gesture: {:?}", gesture);
                        gesture.run();
                        break;
                    }
                }
            }

            tracing::debug!("remove active touch");
            self.active_touches = 0;
            self.swipe = None;
        }
    }
}
