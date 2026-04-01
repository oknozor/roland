use std::{
    process::Child,
    time::{Duration, Instant},
};

use thiserror::Error;

use crate::config::GesturesConfig;

const MAX_FINGERS: usize = 4;

#[derive(Error, Debug)]
pub enum GestureError {
    #[error("no active touches/fingers")]
    NoActiveTouch,
    #[error("no matching gesture found")]
    NoGestureMatched,
    #[error("failed to execute gesture command: {0}")]
    CommandFailed(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct TouchTrace {
    slot: Option<u32>,
    init_pos: (f64, f64),
    current_pos: (f64, f64),
    start_time: Instant,
}

impl TouchTrace {
    pub fn new(slot: Option<u32>, x: f64, y: f64) -> Self {
        Self {
            slot,
            init_pos: (x, y),
            current_pos: (x, y),
            start_time: Instant::now(),
        }
    }

    pub fn current_distance_to(&self, x: f64, y: f64) -> f64 {
        (x - self.current_pos.0).hypot(y - self.current_pos.1)
    }

    pub fn init_distance_to(&self, x: f64, y: f64) -> f64 {
        (x - self.init_pos.0).hypot(y - self.init_pos.1)
    }
}

#[derive(Debug)]
pub struct GestureState {
    traces: Vec<TouchTrace>,
    config: GesturesConfig,
    screen_dimensions: (f64, f64),
}

impl GestureState {
    pub fn new(config: GesturesConfig, screen_width: f64, screen_height: f64) -> Self {
        Self {
            traces: Vec::with_capacity(MAX_FINGERS),
            config,
            screen_dimensions: (screen_width, screen_height),
        }
    }

    pub fn update(&mut self, slot: Option<u32>, x: f64, y: f64) {
        for t in self.traces.iter_mut().filter(|t| t.slot == slot) {
            t.current_pos = (x, y);
        }

        self.handle_gesture();
    }

    pub fn touch_down(&mut self, slot: Option<u32>, x: f64, y: f64) {
        self.traces.push(TouchTrace::new(slot, x, y))
    }

    pub fn touch_up(&mut self, slot: Option<u32>) {
        self.handle_gesture();
        self.traces.retain(|t| t.slot != slot);
    }

    fn init_centeroid(&self) -> Option<(f64, f64)> {
        if self.traces.is_empty() {
            return None;
        };

        let n = self.traces.len() as f64;
        let sum_x: f64 = self.traces.iter().map(|t| t.init_pos.0).sum();
        let sum_y: f64 = self.traces.iter().map(|t| t.init_pos.1).sum();

        Some((sum_x / n, sum_y / n))
    }

    fn current_centroid(&self) -> Option<(f64, f64)> {
        if self.traces.is_empty() {
            return None;
        };

        let n = self.traces.len() as f64;
        let sum_x: f64 = self.traces.iter().map(|t| t.current_pos.0).sum();
        let sum_y: f64 = self.traces.iter().map(|t| t.current_pos.1).sum();

        Some((sum_x / n, sum_y / n))
    }

    fn init_spread(&self) -> Option<f64> {
        let n = self.traces.len() as f64;
        self.init_centeroid().map(|(cx, cy)| {
            self.traces
                .iter()
                .map(|t| t.init_distance_to(cx, cy))
                .sum::<f64>()
                / n
        })
    }

    fn current_spread(&self) -> Option<f64> {
        let n = self.traces.len() as f64;
        self.current_centroid().map(|(cx, cy)| {
            self.traces
                .iter()
                .map(|t| t.current_distance_to(cx, cy))
                .sum::<f64>()
                / n
        })
    }

    fn dispatch(&self) -> Result<Child, GestureError> {
        let spread_diff = self
            .current_spread()
            .zip(self.init_spread())
            .map(|(n, o)| n - o);

        let centroid_diff = self
            .current_centroid()
            .zip(self.init_centeroid())
            .map(|((xn, yn), (xo, yo))| (xn - xo, yn - yo));

        tracing::debug!("Spread: {spread_diff:?}, Centroid: {centroid_diff:?}");

        for c in &self.config.gestures {
            let init_centeroid = self.init_centeroid().ok_or(GestureError::NoActiveTouch)?;
            let longest_duration = self.longest_duration().ok_or(GestureError::NoActiveTouch)?;

            if c.should_trigger(
                self.traces.len(),
                spread_diff,
                centroid_diff,
                init_centeroid,
                self.screen_dimensions,
                longest_duration,
            ) {
                tracing::debug!("Gesture {c:?} concluded");
                match c.run() {
                    Ok(child) => return Ok(child),
                    Err(e) => {
                        tracing::error!("Failed to run command: {e}");
                        return Err(GestureError::CommandFailed(e.into()));
                    }
                }
            }
        }
        Err(GestureError::NoGestureMatched)
    }

    fn handle_gesture(&mut self) {
        match self.dispatch() {
            Ok(_) => self.traces.clear(),
            Err(GestureError::CommandFailed(e)) => {
                tracing::error!("Failed to run gesture command: {e}");
                self.traces.clear();
            }
            _ => {}
        }
    }

    fn longest_duration(&self) -> Option<Duration> {
        self.traces
            .iter()
            .map(|trace| Instant::now().saturating_duration_since(trace.start_time))
            .max()
    }
}
