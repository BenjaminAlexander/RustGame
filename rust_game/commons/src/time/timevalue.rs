use serde::{Deserialize, Serialize};
use std::time::{UNIX_EPOCH, SystemTime};
use std::ops::Add;
use core::time::Duration;
use crate::time::TimeDuration;

pub const EPOCH: TimeValue = TimeValue::from_seconds_since_epoch(0.0);

//TODO: use nanos
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct TimeValue {
    seconds_since_epoch: f64
}

impl TimeValue {
    pub fn now() -> Self {
        return Self {
            seconds_since_epoch: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64()
        }
    }

    pub const fn from_seconds_since_epoch(seconds_since_epoch: f64) -> Self {
        return Self { seconds_since_epoch };
    }

    pub fn get_seconds_since_epoch(&self) -> f64 {
        self.seconds_since_epoch
    }

    pub fn is_after(&self, other: &TimeValue) -> bool {
        self.seconds_since_epoch > other.seconds_since_epoch
    }

    pub fn is_before(&self, other: &TimeValue) -> bool {
        self.seconds_since_epoch < other.seconds_since_epoch
    }

    pub fn add(&self, time_duration: TimeDuration) -> Self {
        return Self {
            seconds_since_epoch: self.seconds_since_epoch + time_duration.get_seconds()
        };
    }

    pub fn subtract(&self, time_duration: TimeDuration) -> Self {
        return Self {
            seconds_since_epoch: self.seconds_since_epoch - time_duration.get_seconds()
        };
    }

    pub fn to_system_time(&self) -> SystemTime {
        return UNIX_EPOCH.add(Duration::from_secs_f64(self.seconds_since_epoch));
    }

    pub fn duration_since(&self, time_before: &TimeValue) -> TimeDuration {
        return TimeDuration::from_seconds(self.seconds_since_epoch - time_before.seconds_since_epoch);
    }
}

impl Into<f64> for TimeValue {
    fn into(self) -> f64 {
        return self.seconds_since_epoch;
    }
}

impl From<f64> for TimeValue {
    fn from(value: f64) -> Self {
        return TimeValue::from_seconds_since_epoch(value);
    }
}