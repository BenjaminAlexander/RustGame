use serde::{
    Deserialize,
    Serialize,
};
use std::cmp::Ordering;
use std::ops::{
    Add,
    Sub,
};
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct TimeDuration {
    seconds: i64,
    nanos: i32,
}

impl TimeDuration {
    pub const NANOS_PER_SEC: i32 = 1_000_000_000;
    pub const ONE_SECOND: TimeDuration = TimeDuration::new(1, 0);

    const fn debug_assert(self) {
        debug_assert!(
            self.seconds.signum() == 0
                || self.nanos.signum() == 0
                || self.seconds.signum() == self.nanos.signum() as i64
        );

        debug_assert!(self.nanos.abs() < TimeDuration::NANOS_PER_SEC);
    }

    const fn new_with_assertions(seconds: i64, nanos: i32) -> TimeDuration {
        let time_duration = Self { seconds, nanos };

        time_duration.debug_assert();

        return time_duration;
    }

    pub const fn new(seconds: i64, nanos: i32) -> TimeDuration {
        let mut seconds: i64 = seconds + ((nanos / TimeDuration::NANOS_PER_SEC) as i64);
        let mut nanos: i32 = nanos % (TimeDuration::NANOS_PER_SEC as i32);

        if seconds.signum() != 0 && nanos.signum() != 0 && seconds.signum() != nanos.signum() as i64
        {
            let original_seconds_signum = seconds.signum();
            seconds = seconds - original_seconds_signum;
            nanos = (TimeDuration::NANOS_PER_SEC as i32 - nanos.abs())
                * (original_seconds_signum as i32);
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
        let nanos = ((value - seconds as f64) * TimeDuration::NANOS_PER_SEC as f64).round() as i32;

        return Self::new_with_assertions(seconds, nanos);
    }

    pub fn as_secs_f64(&self) -> f64 {
        return (self.seconds as f64) + (self.nanos as f64) / (TimeDuration::NANOS_PER_SEC as f64);
    }

    pub fn from_millis_f64(millis: f64) -> Self {
        return Self::from_secs_f64(millis / 1000.0);
    }

    pub fn mul_f64(self, rhs: f64) -> Self {
        return Self::from_secs_f64(rhs * self.as_secs_f64());
    }

    pub fn div_f64(self, rhs: f64) -> Self {
        return Self::from_secs_f64(self.as_secs_f64() / rhs);
    }

    pub fn to_duration(&self) -> Option<Duration> {
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

impl From<&Duration> for TimeDuration {
    fn from(duration: &Duration) -> TimeDuration {
        return TimeDuration::new_with_assertions(
            duration.as_secs() as i64,
            duration.subsec_nanos() as i32,
        );
    }
}

impl Sub<&Self> for TimeDuration {
    type Output = Self;

    fn sub(self, rhs: &Self) -> Self::Output {
        return Self::new(self.seconds - rhs.seconds, self.nanos - rhs.nanos);
    }
}

impl Sub<Self> for &TimeDuration {
    type Output = TimeDuration;

    fn sub(self, rhs: Self) -> Self::Output {
        return TimeDuration::new(self.seconds - rhs.seconds, self.nanos - rhs.nanos);
    }
}

impl Add<&Self> for TimeDuration {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        return Self::new(self.seconds + rhs.seconds, self.nanos + rhs.nanos);
    }
}

impl Add<Self> for &TimeDuration {
    type Output = TimeDuration;

    fn add(self, rhs: Self) -> Self::Output {
        return TimeDuration::new(self.seconds + rhs.seconds, self.nanos + rhs.nanos);
    }
}

impl Eq for TimeDuration {}

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
            result => result,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_time_duration(seconds: i64, nanos: i32, time_duration: TimeDuration) {
        assert_eq!(seconds, time_duration.as_secs());
        assert_eq!(nanos, time_duration.subsec_nanos());
    }

    #[test]
    fn time_duration_test() {
        let time_duration = TimeDuration::new(23, 1_750_000_000);
        assert_time_duration(24, 750_000_000, time_duration);
        assert_eq!(time_duration.as_secs_f64(), 24.75);

        let time_duration = TimeDuration::new(23, -1_750_000_000);
        assert_time_duration(21, 250_000_000, time_duration);
        assert_eq!(time_duration.as_secs_f64(), 21.25);

        let time_duration = TimeDuration::new(1, -1_750_000_000);
        assert_time_duration(0, -750_000_000, time_duration);
        assert_eq!(time_duration.as_secs_f64(), -0.75);

        let time_duration = TimeDuration::new(-23, -1_750_000_000);
        assert_time_duration(-24, -750_000_000, time_duration);
        assert_eq!(time_duration.as_secs_f64(), -24.75);

        let time_duration = TimeDuration::from_secs_f64(1.5);
        assert_time_duration(1, 500_000_000, time_duration);

        let time_duration = TimeDuration::from_millis_f64(500.5);
        assert_time_duration(0, 500_500_000, time_duration);

        let time_duration = TimeDuration::new(0, 500_000_000).mul_f64(2.5);
        assert_time_duration(1, 250_000_000, time_duration);

        let time_duration = TimeDuration::new(1, 0).div_f64(2.5);
        assert_time_duration(0, 400_000_000, time_duration);

        let time_duration = TimeDuration::new(23, 750_000_000);
        let duration = time_duration.to_duration().unwrap();
        assert_eq!(23, duration.as_secs());
        assert_eq!(750_000_000, duration.subsec_nanos());

        let time_duration = TimeDuration::new(-23, 750_000_000);
        assert_eq!(None, time_duration.to_duration());

        let time_duration = TimeDuration::new(23, 0);
        assert_eq!(true, time_duration.is_positive());
        assert_eq!(false, time_duration.is_negetive());

        let time_duration = TimeDuration::new(0, 750_000_000);
        assert_eq!(true, time_duration.is_positive());
        assert_eq!(false, time_duration.is_negetive());

        let time_duration = TimeDuration::new(0, -750_000_000);
        assert_eq!(false, time_duration.is_positive());
        assert_eq!(true, time_duration.is_negetive());

        let time_duration = TimeDuration::new(-1, 0);
        assert_eq!(false, time_duration.is_positive());
        assert_eq!(true, time_duration.is_negetive());

        let time_duration = TimeDuration::new(0, 0);
        assert_eq!(false, time_duration.is_positive());
        assert_eq!(false, time_duration.is_negetive());

        let duration = Duration::new(23, 750_000_000);
        let time_duration = TimeDuration::from(&duration);
        assert_time_duration(23, 750_000_000, time_duration);

        let time_duration1 = TimeDuration::new(23, 750_000_000);
        let time_duration2 = TimeDuration::new(1, 500_000_000);
        assert_time_duration(25, 250_000_000, time_duration1.add(&time_duration2));

        let time_duration1 = TimeDuration::new(23, 750_000_000);
        let time_duration2 = TimeDuration::new(-1, -500_000_000);
        assert_time_duration(22, 250_000_000, time_duration1.add(&time_duration2));

        let time_duration1 = TimeDuration::new(23, 750_000_000);
        let time_duration2 = TimeDuration::new(1, 500_000_000);
        assert_time_duration(22, 250_000_000, time_duration1.sub(&time_duration2));

        let time_duration1 = TimeDuration::new(23, 750_000_000);
        let time_duration2 = TimeDuration::new(-1, -500_000_000);
        assert_time_duration(25, 250_000_000, time_duration1.sub(&time_duration2));

        let time_duration1 = TimeDuration::new(23, 750_000_000);
        let time_duration2 = TimeDuration::new(23, 500_000_000);
        assert_eq!(false, time_duration1.eq(&time_duration2));

        let time_duration1 = TimeDuration::new(23, 750_000_000);
        let time_duration2 = TimeDuration::new(22, 750_000_000);
        assert_eq!(false, time_duration1.eq(&time_duration2));

        let time_duration1 = TimeDuration::new(23, 750_000_000);
        let time_duration2 = TimeDuration::new(23, 750_000_000);
        assert_eq!(true, time_duration1.eq(&time_duration2));

        let time_duration1 = TimeDuration::new(23, 750_000_000);
        let time_duration2 = TimeDuration::new(22, 750_000_000);
        assert_eq!(
            Some(Ordering::Greater),
            time_duration1.partial_cmp(&time_duration2)
        );

        let time_duration1 = TimeDuration::new(23, 750_000_000);
        let time_duration2 = TimeDuration::new(23, 500_000_000);
        assert_eq!(
            Some(Ordering::Greater),
            time_duration1.partial_cmp(&time_duration2)
        );

        assert_time_duration(0, 82_921_124, TimeDuration::new(1, -917_078_876));
    }
}
