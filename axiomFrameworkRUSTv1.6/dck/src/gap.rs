use std::collections::VecDeque;

use chrono::{DateTime, Utc};

pub struct GapHistory {
    history: VecDeque<(DateTime<Utc>, f64)>,
    capacity: usize,
    tau: f64,
    smoothed_velocity: f64,
}

impl GapHistory {
    pub fn new(capacity: usize, tau: f64) -> Self {
        Self {
            history: VecDeque::with_capacity(capacity),
            capacity,
            tau: tau.max(1e-6),
            smoothed_velocity: 0.0,
        }
    }

    /// Push absolute gap and return smoothed velocity (positive = gap closing).
    pub fn push(&mut self, timestamp: DateTime<Utc>, gap: f64) -> f64 {
        if self.history.is_empty() {
            self.history.push_back((timestamp, gap));
            return 0.0;
        }

        let (prev_ts, prev_gap) = *self.history.back().unwrap();
        let dt = (timestamp - prev_ts).num_milliseconds() as f64 / 1000.0;

        if self.history.len() >= self.capacity {
            self.history.pop_front();
        }
        self.history.push_back((timestamp, gap));

        if dt <= 1e-6 {
            return self.smoothed_velocity;
        }

        let raw_velocity = (prev_gap - gap) / dt;
        let alpha = 1.0 - (-dt / self.tau).exp();
        self.smoothed_velocity = alpha * raw_velocity + (1.0 - alpha) * self.smoothed_velocity;
        self.smoothed_velocity
    }
}
