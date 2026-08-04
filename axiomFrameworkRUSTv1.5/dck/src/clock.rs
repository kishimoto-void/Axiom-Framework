use chrono::{DateTime, Utc};

/// Injectable clock for testability.
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    #[inline]
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
