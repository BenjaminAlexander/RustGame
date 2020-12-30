use std::time::{Instant, UNIX_EPOCH, SystemTime, Duration};
use std::ops::{Div, Mul};

#[derive(Clone, Copy, Debug)]
pub struct TimeDuration {
    millis: i64
}

impl TimeDuration {
    pub fn from_millis(millis: i64) -> TimeDuration {
        TimeDuration{millis}
    }

    pub fn get_millis(&self) -> i64 {
        self.millis
    }

    pub fn to_std(&self) -> Duration {
        Duration::from_millis(self.millis.abs() as u64)
    }
}

impl From<Duration> for TimeDuration {
    fn from(duration: Duration) -> Self {
        Self::from_millis(duration.as_millis() as i64)
    }
}

impl Div<TimeDuration> for TimeDuration {
    type Output = f64;

    fn div(self, rhs: TimeDuration) -> Self::Output {
        self.millis as f64 / rhs.millis as f64
    }
}

impl<T> Mul<T> for TimeDuration
    where i64: Mul<T, Output = i64> {
    type Output = TimeDuration;

    fn mul(self, rhs: T) -> Self::Output {
        TimeDuration{millis: self.millis.mul(rhs)}
    }
}