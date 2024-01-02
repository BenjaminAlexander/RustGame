use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::ops::{Sub, Add};
use std::cmp::Ordering;

pub const NANOS_PER_SEC: i32 = 1_000_000_000;

//Time In Milliseconds
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct TimeDuration {
    seconds: i64,
    nanos: i32
}

impl TimeDuration {

    const fn debug_assert(self) {

        debug_assert!(
            self.seconds.signum() == 0 || 
            self.nanos.signum() == 0 || 
            self.seconds.signum() == self.nanos.signum() as i64);
            
        debug_assert!(self.nanos.abs() < NANOS_PER_SEC);
    }

    const fn new_with_assertions(seconds: i64, nanos: i32) -> TimeDuration {

        let time_duration = Self {
            seconds,
            nanos
        };

        time_duration.debug_assert();

        return time_duration;
    }

    pub const fn new(seconds: i64, nanos: i32) -> TimeDuration {

        let mut seconds: i64 = seconds + ((nanos / NANOS_PER_SEC) as i64);
        let mut nanos: i32 = nanos % (NANOS_PER_SEC as i32);

        if seconds.signum() != 0 && 
                nanos.signum() != 0 && 
                seconds.signum() != nanos.signum() as i64 {

            seconds = seconds - seconds.signum();
            nanos = (NANOS_PER_SEC as i32 - nanos.abs()) * (seconds.signum() as i32);
        }
        
        return Self::new_with_assertions(seconds, nanos);
    }

    pub const fn subsec_nanos(self) -> i32 {
        return self.nanos;
    }

    pub const fn as_secs(self) -> i64 {
        return self.seconds;
    }

    pub fn from_secs_f64(value: f64) -> Self {

        let seconds = value.trunc() as i64;
        let nanos = ((value - seconds as f64) / NANOS_PER_SEC as f64) as i32;

        return Self::new_with_assertions(seconds, nanos);
    }

    pub fn as_secs_f64(&self) -> f64 {
        return (self.seconds as f64) + (self.nanos as f64) / (NANOS_PER_SEC as f64);
    }

    pub fn from_millis(millis: f64) -> Self {
        return Self::from_secs_f64(millis / 1000.0);
    }

    pub fn mul_f64(self, rhs: f64) -> Self {
        return Self::from_secs_f64(rhs * self.as_secs_f64());
    }

    pub fn div_f64(self, rhs: f64) -> Self {
        return Self::from_secs_f64(self.as_secs_f64() / rhs);
    }

    pub fn one_second() -> Self {
        return Self::from_secs_f64(1.0);
    }

    pub fn to_std(&self) -> Option<Duration> {

        self.debug_assert();

        if !self.seconds.is_negative() && !self.nanos.is_negative() {
            return Some(Duration::new(self.seconds as u64, self.nanos as u32));
        } else {
            return None;
        }
    }

    pub fn is_positive(&self) -> bool {
        return self.seconds.is_positive() || self.nanos.is_positive();
    }

    pub fn is_negetive(&self) -> bool {
        return self.seconds.is_negative() || self.nanos.is_negative();
    }
}

impl From<Duration> for TimeDuration {
    fn from(duration: Duration) -> TimeDuration {
        return TimeDuration::new_with_assertions(duration.as_secs() as i64, duration.subsec_nanos() as i32);
    }
}

impl From<TimeDuration> for Duration {
    fn from(time_duration: TimeDuration) -> Duration {
        return Duration::new(time_duration.seconds as u64, time_duration.nanos as u32);
    }
}

impl Sub for TimeDuration {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        return Self::new(self.seconds - rhs.seconds, self.nanos - rhs.nanos);
    }
}

impl Add for TimeDuration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        return Self::new(self.seconds + rhs.seconds, self.nanos + rhs.nanos);
    }
}

impl Eq for TimeDuration {
}

impl PartialEq for TimeDuration {
    fn eq(&self, other: &TimeDuration) -> bool {

        self.debug_assert();
        other.debug_assert();

        return self.seconds.eq(&other.seconds) && self.nanos.eq(&other.nanos);
    }
}

impl PartialOrd for TimeDuration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(other));
    }
}


impl Ord for TimeDuration {
    fn cmp(&self, other: &Self) -> Ordering {

        self.debug_assert();
        other.debug_assert();

        return match self.seconds.cmp(&other.seconds) {
            Ordering::Equal => self.nanos.cmp(&other.nanos),
            result => result
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::time::TimeValue;

    use super::*;

    #[test]
    fn timer_value_test() {
        let time_duration = TimeDuration::new(23, 1_750_000_000);

        assert_eq!(24, time_duration.as_secs());
        assert_eq!(750_000_000, time_duration.subsec_nanos());

    }

}
