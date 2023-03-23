use serde::{Deserialize, Serialize};
use std::time::{Duration};
use std::ops::{Div, Mul, Sub, Add};
use std::cmp::Ordering;

//Time In Milliseconds
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct TimeDuration {
    seconds: f64
}

impl TimeDuration {
    pub const fn from_seconds(seconds: f64) -> TimeDuration {
        return Self {
            seconds
        };
    }

    pub fn from_millis(millis: f64) -> TimeDuration {
        return Self::from_seconds(millis / 1000.0);
    }

    pub const fn one_second() -> Self {
        return Self::from_seconds(1.0);
    }

    pub fn get_seconds(&self) -> f64 {
        return self.seconds;
    }

    pub fn to_std(&self) -> Duration {
        return Duration::from_secs_f64(self.seconds);
    }

    pub fn is_positive(&self) -> bool {
        return self.seconds > 0.0;
    }

    pub fn is_negetive(&self) -> bool {
        return self.seconds < 0.0;
    }
}

impl From<Duration> for TimeDuration {
    fn from(duration: Duration) -> Self {
        Self::from_seconds(duration.as_secs_f64())
    }
}

impl Sub for TimeDuration {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        return Self {
            seconds: self.seconds - rhs.seconds
        };
    }
}

impl Add for TimeDuration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        return Self {
            seconds: self.seconds + rhs.seconds
        };
    }
}

impl Div for TimeDuration {
    type Output = f64;

    fn div(self, rhs: Self) -> Self::Output {
        return self.seconds / rhs.seconds;
    }
}

impl<T> Mul<T> for TimeDuration
    where f64: Mul<T, Output = f64> {
    type Output = TimeDuration;

    fn mul(self, rhs: T) -> Self::Output {
        return Self {
            seconds: self.seconds * rhs
        };
    }
}

impl PartialEq<TimeDuration> for TimeDuration {
    fn eq(&self, other: &TimeDuration) -> bool {
        return self.seconds.eq(&other.seconds);
    }
}

impl PartialOrd<TimeDuration> for TimeDuration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return self.seconds.partial_cmp(&other.seconds);
    }
}

impl Into<f64> for TimeDuration {
    fn into(self) -> f64 {
        return self.seconds;
    }
}

impl From<f64> for TimeDuration {
    fn from(value: f64) -> Self {
        return TimeDuration::from_seconds(value);
    }
}