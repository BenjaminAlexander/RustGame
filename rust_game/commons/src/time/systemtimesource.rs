use std::time::{SystemTime, UNIX_EPOCH};
use crate::time::{TimeSource, TimeValue};

#[derive(Clone, Copy)]
pub struct SystemTimeSource;

impl Default for SystemTimeSource {
    fn default() -> Self {
        return Self;
    }
}

impl TimeSource for SystemTimeSource {
    fn now(&self) -> TimeValue {
        return TimeValue::from_seconds_since_epoch(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64());
    }
}