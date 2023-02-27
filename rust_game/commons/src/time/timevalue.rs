use serde::{Deserialize, Serialize};
use std::time::{UNIX_EPOCH, SystemTime};
use std::ops::Add;
use core::time::Duration;
use crate::time::TimeDuration;

pub const EPOCH: TimeValue = TimeValue::from_millis(0);

//TODO: use nanos
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct TimeValue {
    millis_since_epoch: i64
}

impl TimeValue {
    pub fn now() -> Self {
        let now = SystemTime::now();
        TimeValue{millis_since_epoch: now.duration_since(UNIX_EPOCH).unwrap().as_millis() as i64 }
    }

    pub const fn from_millis(millis_since_epoch: i64) -> Self {
        TimeValue{millis_since_epoch}
    }

    pub fn get_millis_since_epoch(&self) -> i64 {
        self.millis_since_epoch
    }

    pub fn is_after(&self, other: &TimeValue) -> bool {
        self.millis_since_epoch > other.millis_since_epoch
    }

    pub fn is_before(&self, other: &TimeValue) -> bool {
        self.millis_since_epoch < other.millis_since_epoch
    }

    pub fn add(&self, time_duration: TimeDuration) -> Self {
        TimeValue{millis_since_epoch: self.millis_since_epoch + time_duration.get_millis()}
    }

    pub fn subtract(&self, time_duration: TimeDuration) -> Self {
        TimeValue{millis_since_epoch: self.millis_since_epoch - time_duration.get_millis()}
    }

    pub fn to_system_time(&self) -> SystemTime {
        UNIX_EPOCH.add(Duration::from_millis(self.millis_since_epoch as u64))
    }

    pub fn duration_since(&self, time_before: &TimeValue) -> TimeDuration {
        TimeDuration::from_millis(self.millis_since_epoch - time_before.millis_since_epoch)
    }
}

impl Into<f64> for TimeValue {
    fn into(self) -> f64 {
        return self.get_millis_since_epoch() as f64;
    }
}

impl From<f64> for TimeValue {
    fn from(value: f64) -> Self {
        return TimeValue::from_millis(value as i64);
    }
}