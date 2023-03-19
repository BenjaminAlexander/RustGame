use std::time::{SystemTime, UNIX_EPOCH};
use crate::time::{TimeSource, TimeValue};

pub struct SystemTimeSource;

impl TimeSource for SystemTimeSource {
    fn now() -> TimeValue {
        return TimeValue::from_seconds_since_epoch(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64());
    }
}