use std::time::Instant;

/// Monotonic playback clock (the master clock until audio is wired up).
///
/// Time is expressed in seconds, relative to the media start.
#[derive(Debug)]
pub struct Clock {
    base_pts: f64,
    base_instant: Instant,
    paused: bool,
    speed: f64,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            base_pts: 0.0,
            base_instant: Instant::now(),
            paused: false,
            speed: 1.0,
        }
    }
}

impl Clock {
    pub fn time(&self) -> f64 {
        if self.paused {
            self.base_pts
        } else {
            self.base_pts + self.base_instant.elapsed().as_secs_f64() * self.speed
        }
    }

    pub fn set(&mut self, pts: f64) {
        self.base_pts = pts;
        self.base_instant = Instant::now();
    }

    pub fn set_paused(&mut self, paused: bool) {
        let now = self.time();
        self.paused = paused;
        self.set(now);
    }

    pub fn set_speed(&mut self, speed: f64) {
        let now = self.time();
        self.speed = speed;
        self.set(now);
    }
}
