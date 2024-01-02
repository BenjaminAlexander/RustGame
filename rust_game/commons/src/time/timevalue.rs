use serde::{Deserialize, Serialize};
use std::time::{UNIX_EPOCH, SystemTime};
use std::ops::{Add, Sub};
use core::time::Duration;
use std::cmp::Ordering;
use crate::time::TimeDuration;

pub const EPOCH: TimeValue = TimeValue::new(0, 0);
pub const NANOS_PER_SEC: u32 = 1_000_000_000;

//TODO: use nanos
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct TimeValue {
    seconds_since_epoch: u64,
    nanos: u32 // Always 0 <= nanos < NANOS_PER_SEC
}

impl TimeValue {

    const fn debug_assert(self) {
        debug_assert!(self.nanos < NANOS_PER_SEC);
    }

    pub const fn new(seconds_since_epoch: u64, nanos: u32) -> Self {

        let normalized_seconds = seconds_since_epoch + (nanos / NANOS_PER_SEC) as u64;
        let normalized_nanos = nanos % NANOS_PER_SEC;

        let time_value = Self {
            seconds_since_epoch: normalized_seconds,
            nanos: normalized_nanos
        };

        time_value.debug_assert();

        return time_value;
    }

    const fn from_signed_nanos(seconds_since_epoch: u64, nanos: i32) -> Self {

        let mut normalized_seconds = (seconds_since_epoch as i64 + (nanos as i64 / NANOS_PER_SEC as i64)) as u64;
        let mut normalized_nanos = nanos % NANOS_PER_SEC as i32;

        if normalized_nanos.is_negative() {
            normalized_seconds = normalized_seconds - 1;
            normalized_nanos = NANOS_PER_SEC as i32 - normalized_nanos;
        };

        let time_value = Self {
            seconds_since_epoch: normalized_seconds,
            nanos: normalized_nanos as u32
        };

        time_value.debug_assert();

        return time_value;
    }

    pub fn from_secs_f64(value: f64) -> Self {

        let seconds_since_epoch = value.trunc() as u64;
        let nanos = ((value - seconds_since_epoch as f64) / NANOS_PER_SEC as f64) as u32;

        let time_value = Self {
            seconds_since_epoch,
            nanos
        };

        time_value.debug_assert();

        return time_value;
    }

    pub fn as_secs_f64(&self) -> f64 {
        return (self.seconds_since_epoch as f64) + (self.nanos as f64) / (NANOS_PER_SEC as f64);
    }

    pub fn is_after(&self, other: &TimeValue) -> bool {
        return self > other;
    }

    pub fn is_before(&self, other: &TimeValue) -> bool {
        return self < other;
    }

    pub fn to_system_time(&self) -> SystemTime {
        return UNIX_EPOCH.add(Duration::new(self.seconds_since_epoch, self.nanos));
    }

    pub fn duration_since(&self, time_before: &TimeValue) -> TimeDuration {

        let seconds = self.seconds_since_epoch as i64 - time_before.seconds_since_epoch as i64;
        let nanos = self.nanos as i32 - time_before.nanos as i32;
        return TimeDuration::new(seconds, nanos);
    }
}

impl Add<TimeDuration> for TimeValue {
    type Output = Self;

    fn add(self, rhs: TimeDuration) -> Self::Output {

        let seconds_since_epoch = self.seconds_since_epoch as i64 + rhs.as_secs();
        let nanos = self.nanos as i32 + rhs.subsec_nanos();

        return Self::from_signed_nanos(seconds_since_epoch as u64, nanos);
    }
}

impl Sub<TimeDuration> for TimeValue {
    type Output = Self;

    fn sub(self, rhs: TimeDuration) -> Self::Output {

        let seconds_since_epoch = self.seconds_since_epoch as i64 - rhs.as_secs();
        let nanos = self.nanos as i32 - rhs.subsec_nanos();

        return Self::from_signed_nanos(seconds_since_epoch as u64, nanos);
    }
}

impl From<&TimeValue> for f64 {
    fn from(value: &TimeValue) -> f64 {
        return Duration::new(value.seconds_since_epoch, value.nanos).as_secs_f64();
    }
}

impl From<f64> for TimeValue {
    fn from(value: f64) -> Self {
        return TimeValue::from(Duration::from_secs_f64(value));
    }
}

impl From<SystemTime> for TimeValue {
    fn from(value: SystemTime) -> Self {
        return TimeValue::from(value.duration_since(UNIX_EPOCH).unwrap());
    }
}

impl From<Duration> for TimeValue {
    fn from(value: Duration) -> Self {
        return TimeValue::new(value.as_secs(), value.subsec_nanos());
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
        return Some(self.cmp(other));
    }
}

impl Ord for TimeValue {
    fn cmp(&self, other: &Self) -> Ordering {

        self.debug_assert();
        other.debug_assert();

        return match self.seconds_since_epoch.cmp(&other.seconds_since_epoch) {
            Ordering::Equal => self.nanos.cmp(&other.nanos),
            result => result
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_value_test() {
        //let time_value1 = TimeValue::from_seconds_since_epoch(100.0);
        //let time_value2 = TimeValue::from_seconds_since_epoch(200.0);

        //assert_eq!(true, time_value1.is_before(&time_value2));
        //assert_eq!(true, time_value2.is_after(&time_value1));

        //SystemTime::now();
    }

}