use serde::{Deserialize, Serialize};
use std::time::{Duration};
use std::ops::{Div, Mul, Sub, Add};
use std::cmp::Ordering;

//Time In Milliseconds
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct TimeDuration(pub i64);

impl TimeDuration {
    pub const fn from_millis(millis: i64) -> TimeDuration {
        TimeDuration(millis)
    }

    pub const fn one_second() -> Self {
        return Self::from_millis(1000);
    }

    pub fn get_millis(&self) -> i64 {
        self.0
    }

    pub fn to_std(&self) -> Duration {
        Duration::from_millis(self.0.abs() as u64)
    }
}

impl From<Duration> for TimeDuration {
    fn from(duration: Duration) -> Self {
        Self::from_millis(duration.as_millis() as i64)
    }
}

impl Sub for TimeDuration {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        return Self (self.0 - rhs.0);
    }
}

impl Add for TimeDuration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        return Self (self.0 + rhs.0);
    }
}

impl Div for TimeDuration {
    type Output = f64;

    fn div(self, rhs: Self) -> Self::Output {
        self.0 as f64 / rhs.0 as f64
    }
}

impl<T> Mul<T> for TimeDuration
    where i64: Mul<T, Output = i64> {
    type Output = TimeDuration;

    fn mul(self, rhs: T) -> Self::Output {
        TimeDuration(self.0.mul(rhs))
    }
}

impl PartialEq<TimeDuration> for TimeDuration {
    fn eq(&self, other: &TimeDuration) -> bool {
        self.0.eq(&other.0)
    }
}

impl PartialOrd<TimeDuration> for TimeDuration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Into<f64> for TimeDuration {
    fn into(self) -> f64 {
        return self.0 as f64;
    }
}

impl From<f64> for TimeDuration {
    fn from(value: f64) -> Self {
        return TimeDuration(value as i64);
    }
}