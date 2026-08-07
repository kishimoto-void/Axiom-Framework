//! Injectable clock for determinism and testability.

use chrono::{DateTime, Duration, Utc};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

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

/// Deterministic clock for Golden Vector / unit tests.
#[derive(Debug)]
pub struct MockClock {
    offset_ms: AtomicI64,
    base: DateTime<Utc>,
}

impl MockClock {
    pub fn new(base: DateTime<Utc>) -> Self {
        Self { offset_ms: AtomicI64::new(0), base }
    }

    pub fn at_unix_ms(ms: i64) -> Self {
        let base = DateTime::<Utc>::from_timestamp_millis(ms).unwrap_or_else(|| Utc::now());
        Self::new(base)
    }

    pub fn advance_ms(&self, ms: i64) {
        self.offset_ms.fetch_add(ms, Ordering::SeqCst);
    }

    pub fn advance(&self, d: Duration) {
        self.advance_ms(d.num_milliseconds());
    }

    pub fn into_arc(self) -> Arc<dyn Clock> {
        Arc::new(self)
    }
}

impl Clock for MockClock {
    fn now(&self) -> DateTime<Utc> {
        let off = self.offset_ms.load(Ordering::SeqCst);
        self.base + Duration::milliseconds(off)
    }
}

impl Default for MockClock {
    fn default() -> Self {
        let base = DateTime::<Utc>::from_timestamp(1_700_000_000, 0).unwrap_or_else(|| Utc::now());
        Self::new(base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_clock_is_deterministic() {
        let c = MockClock::default();
        let t1 = c.now();
        let t2 = c.now();
        assert_eq!(t1, t2);
        c.advance_ms(1000);
        let t3 = c.now();
        assert!(t3 > t1);
        assert_eq!((t3 - t1).num_milliseconds(), 1000);
    }
}
