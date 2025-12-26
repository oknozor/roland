use std::time::Duration;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GesturesConfig {
    pub gestures: Vec<GestureConfig>,
}

#[derive(Deserialize, Debug)]
pub struct GestureConfig {
    num_fingers: u32,
    kind: GestureKind,
    action: String,
    min_duration: Option<u64>,
    max_duration: Option<u64>,
    max_distance: Option<f64>,
    min_distance: Option<f64>,
    on_edge: Option<EdgeRequirement>,
}

impl GestureConfig {
    pub fn run(&self) {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(&self.action)
            .spawn()
            .expect("Failed to execute command");
    }

    pub fn should_trigger(
        &self,
        num_finger: u32,
        start_position: (f64, f64),
        current_position: (f64, f64),
        screen_size: (f64, f64),
        duration: Duration,
    ) -> bool {
        if num_finger != self.num_fingers {
            tracing::debug!("Gesture finger count mismatch, ignoring");
            return false;
        }

        if let Some(min_duration) = self.min_duration {
            if duration < Duration::from_millis(min_duration) {
                tracing::debug!("Gesture duration below min_duration, ignoring");
                return false;
            }
        }

        if let Some(max_duration) = self.max_duration {
            if duration > Duration::from_millis(max_duration) {
                tracing::debug!("Gesture duration exceeds max_duration, ignoring");
                return false;
            }
        }

        let dx = current_position.0 - start_position.0;
        let dy = current_position.1 - start_position.1;
        let distance = ((dx * dx + dy * dy) as f64).sqrt();

        if let Some(min_distance) = self.min_distance {
            if distance < min_distance {
                tracing::debug!("Gesture distance below min_distance, ignoring");
                return false;
            }
        }

        if let Some(max_distance) = self.max_distance {
            if distance > max_distance {
                tracing::debug!("Gesture distance exceeds max_distance, ignoring");
                return false;
            }
        }

        if !self.is_on_screen_edge(
            start_position.0 as f64,
            start_position.1 as f64,
            screen_size.0 as f64,
            screen_size.1 as f64,
        ) {
            tracing::debug!("Gesture not on screen edge, ignoring");
            return false;
        }

        match self.kind {
            GestureKind::SwipeUp => {
                if is_vertical(dx, dy) && dy < 0.0 {
                    tracing::debug!("Gesture swipe up detected");
                    return true;
                }
            }
            GestureKind::SwipeDown => {
                if is_vertical(dx, dy) && dy > 0.0 {
                    tracing::debug!("Gesture swipe down detected");
                    return true;
                }
            }
            GestureKind::SwipeLeft => {
                if is_horizontal(dx, dy) && dx < 0.0 {
                    tracing::debug!("Gesture swipe left detected");
                    return true;
                }
            }
            GestureKind::SwipeRight => {
                if is_horizontal(dx, dy) && dx > 0.0 {
                    tracing::debug!("Gesture swipe right detected");
                    return true;
                }
            }
            GestureKind::Press => {
                // Press gesture does not depend on movement
                return true;
            }
        }

        false
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

#[derive(Deserialize, Debug)]
pub enum EdgeRequirement {
    Top(u32),
    Left(u32),
    Bottom(u32),
    Right(u32),
}

#[derive(Deserialize, Debug)]
pub enum GestureKind {
    SwipeUp,
    SwipeDown,
    SwipeLeft,
    SwipeRight,
    Press,
}

impl GesturesConfig {
    pub fn from_path(path: impl AsRef<std::path::Path>) -> color_eyre::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: GesturesConfig = toml::from_str(&content)?;
        Ok(config)
    }
}

fn is_vertical(dx: f64, dy: f64) -> bool {
    dy.abs() > dx.abs()
}

fn is_horizontal(dx: f64, dy: f64) -> bool {
    dx.abs() > dy.abs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_from_path() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("config.toml");

        let config = GesturesConfig::from_path(&path).unwrap();
        println!("{:?}", config)
    }
}
