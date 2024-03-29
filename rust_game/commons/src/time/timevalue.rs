use serde::{Deserialize, Serialize};
use std::time::{UNIX_EPOCH, SystemTime};
use std::ops::Add;
use core::time::Duration;
use std::cmp::Ordering;
use crate::time::TimeDuration;

pub const EPOCH: TimeValue = TimeValue::from_seconds_since_epoch(0.0);

//TODO: use nanos
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct TimeValue {
    seconds_since_epoch: f64
}

impl TimeValue {

    pub const fn from_seconds_since_epoch(seconds_since_epoch: f64) -> Self {
        return Self { seconds_since_epoch };
    }

    pub fn get_seconds_since_epoch(&self) -> f64 {
        self.seconds_since_epoch
    }

    pub fn is_after(&self, other: &TimeValue) -> bool {
        return self > other;
    }

    pub fn is_before(&self, other: &TimeValue) -> bool {
        return self < other;
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

impl PartialEq<Self> for TimeValue {
    fn eq(&self, other: &Self) -> bool {
        return self.seconds_since_epoch.eq(&other.seconds_since_epoch);
    }
}

impl Eq for TimeValue {

}

impl PartialOrd<Self> for TimeValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return self.seconds_since_epoch.partial_cmp(&other.seconds_since_epoch);
    }
}

impl Ord for TimeValue {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.seconds_since_epoch < other.seconds_since_epoch {
            return Ordering::Less;
        } else if self.seconds_since_epoch > other.seconds_since_epoch {
            return Ordering::Greater;
        } else {
            return Ordering::Equal;
        }
    }
}