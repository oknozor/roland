use std::{process::Child, time::Duration};

use serde::Deserialize;

const SIN_PI_8: f64 = 0.38268;

#[derive(Deserialize, Debug)]
pub struct GesturesConfig {
    pub gestures: Vec<GestureConfig>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct GestureConfig {
    num_fingers: usize,
    action: String,
    min_duration: Option<u64>,
    max_duration: Option<u64>,
    min_distance: Option<f64>,
    max_distance: Option<f64>,
    kind: GestureKind,
    on_edge: Option<EdgeRequirement>,
}

impl GestureConfig {
    pub fn run(&self) -> Result<Child, std::io::Error> {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(&self.action)
            .spawn()
    }

    pub fn should_trigger(
        &self,
        num_finger: usize,
        spread_diff: Option<f64>,
        centroid_diff: Option<(f64, f64)>,
        start_position: (f64, f64),
        screen_size: (f64, f64),
        duration: Duration,
    ) -> bool {
        let gestures = self.conclude_gestures(
            num_finger,
            spread_diff,
            centroid_diff,
            start_position,
            screen_size,
            duration,
        );

        if gestures.contains(&self.kind) {
            tracing::debug!(
                "Gesture concluded {num_finger}/{gestures:?} <==> Trigger target {}/{:?}",
                self.num_fingers,
                self.kind
            );
            return true;
        }
        return false;
    }

    fn conclude_gestures(
        &self,
        num_finger: usize,
        spread_diff: Option<f64>,
        centroid_diff: Option<(f64, f64)>,
        start_position: (f64, f64),
        screen_size: (f64, f64),
        duration: Duration,
    ) -> Vec<GestureKind> {
        let mut gestures = Vec::with_capacity(3);

        if num_finger != self.num_fingers {
            return gestures;
        }

        if let Some(min_duration) = self.min_duration
            && duration < Duration::from_millis(min_duration)
        {
            return gestures;
        }

        if let Some(max_duration) = self.max_duration
            && duration > Duration::from_millis(max_duration)
        {
            return gestures;
        }

        if !self.is_on_screen_edge(
            start_position.0,
            start_position.1,
            screen_size.0,
            screen_size.1,
        ) {
            return gestures;
        }

        let min_distance = self.min_distance.unwrap_or(0.0);

        let (dx, dy) = match centroid_diff {
            Some(diff) => diff,
            None => return gestures,
        };
        let dr = dx.hypot(dy);

        if dr < min_distance {
            return gestures;
        }

        if let Some(max_distance) = self.max_distance
            && dr > max_distance
        {
            return gestures;
        }

        gestures.push(GestureKind::Hold);

        match spread_diff {
            Some(ds) if ds > min_distance => gestures.push(GestureKind::PinchOut),
            Some(ds) if ds < -min_distance => gestures.push(GestureKind::PinchIn),
            _ => {}
        }

        let projected = dx.hypot(dy) * SIN_PI_8;
        match (
            dx > projected,
            dx < -projected,
            dy > projected,
            dy < -projected,
        ) {
            (true, false, false, false) => gestures.push(GestureKind::SwipeRight),
            (false, true, false, false) => gestures.push(GestureKind::SwipeLeft),
            (false, false, true, false) => gestures.push(GestureKind::SwipeDown),
            (false, false, false, true) => gestures.push(GestureKind::SwipeUp),

            (true, false, true, false) => gestures.push(GestureKind::SwipeDownRight),
            (false, true, true, false) => gestures.push(GestureKind::SwipeDownLeft),
            (true, false, false, true) => gestures.push(GestureKind::SwipeUpRight),
            (false, true, false, true) => gestures.push(GestureKind::SwipeUpLeft),

            _ => {}
        };

        return gestures;
    }

    fn is_on_screen_edge(&self, x: f64, y: f64, screen_width: f64, screen_height: f64) -> bool {
        match self.on_edge {
            Some(EdgeRequirement::Top(size)) => y <= size as f64,
            Some(EdgeRequirement::Left(size)) => x <= size as f64,
            Some(EdgeRequirement::Bottom(size)) => y >= (screen_height - size as f64),
            Some(EdgeRequirement::Right(size)) => x >= (screen_width - size as f64),
            None => true,
        }
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum EdgeRequirement {
    Top(u32),
    Left(u32),
    Bottom(u32),
    Right(u32),
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum GestureKind {
    SwipeUp,
    SwipeDown,
    SwipeLeft,
    SwipeRight,
    SwipeUpRight,
    SwipeUpLeft,
    SwipeDownRight,
    SwipeDownLeft,
    PinchIn,
    PinchOut,
    Hold,
}

impl GesturesConfig {
    pub fn from_path(path: impl AsRef<std::path::Path>) -> color_eyre::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: GesturesConfig = toml::from_str(&content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_from_path() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("config.example.toml");

        let config = GesturesConfig::from_path(&path).unwrap();
        println!("{:?}", config)
    }
}
