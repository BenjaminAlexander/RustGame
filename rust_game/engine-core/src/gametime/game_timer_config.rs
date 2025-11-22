use commons::time::{TimeDuration, TimeValue};
use serde::{
    Deserialize,
    Serialize,
};
use std::ops::{Add, Sub};

// TODO: rename file

#[derive(Serialize, Deserialize, Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FrameIndex(usize);

impl FrameIndex {
    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn next(&self) -> FrameIndex {
        Self(self.0 + 1) 
    }

    pub const fn usize(&self) -> usize {
        self.0
    }
}

impl From<usize> for FrameIndex {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Into<usize> for FrameIndex {
    fn into(self) -> usize {
        self.0
    }
}

impl Into<usize> for &FrameIndex {
    fn into(self) -> usize {
        self.0
    }
}

impl Into<f64> for FrameIndex {
    fn into(self) -> f64 {
        self.0 as f64
    }
}

impl Into<f64> for &FrameIndex {
    fn into(self) -> f64 {
        self.0 as f64
    }
}

impl Add<usize> for &FrameIndex {
    type Output = FrameIndex;

    fn add(self, rhs: usize) -> Self::Output {
        FrameIndex(self.0 + rhs)
    }
}

impl Add<usize> for FrameIndex {
    type Output = FrameIndex;

    fn add(self, rhs: usize) -> Self::Output {
        FrameIndex(self.0 + rhs)
    }
}

impl Sub<usize> for &FrameIndex {
    type Output = FrameIndex;

    fn sub(self, rhs: usize) -> Self::Output {
        FrameIndex(self.0 - rhs)
    }
}

impl Sub<usize> for FrameIndex {
    type Output = FrameIndex;

    fn sub(self, rhs: usize) -> Self::Output {
        FrameIndex(self.0 - rhs)
    }
}

/// The [`TimeDuration`] between frames
#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct FrameDuration (TimeDuration);

impl FrameDuration {
    pub fn new(frame_duration: TimeDuration) -> Self {
        Self(frame_duration)
    }

    pub fn get_frame_duration(&self) -> &TimeDuration {
        &self.0
    }

    pub fn duration_from_start(&self, frame_index: &FrameIndex) -> TimeDuration {
        self.0.mul_f64(frame_index.0 as f64)
    }

    pub fn to_frame_count(&self, time_duration: &TimeDuration) -> f64 {
        time_duration.as_secs_f64() / self.0.as_secs_f64()
    }

    //TODO: get FrameIndex
}

/// The time of occurance for frame index 0.  On clients, this time can float around to slave the client's clock to the server's clock.
#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct StartTime(TimeValue);

impl StartTime {
    pub(super) fn new(start_time: TimeValue) -> Self {
        Self(start_time)
    }

    pub fn get_frame_time_of_occurence(&self, frame_duration: &FrameDuration, frame_index: &FrameIndex) -> TimeValue {
        self.0.add(frame_duration.duration_from_start(frame_index))
    }

    pub fn get_fractional_frame_index(&self, frame_duration: &FrameDuration, time_value: &TimeValue) -> f64 {
        let duration_since_start = time_value.duration_since(&self.0);
        duration_since_start.as_secs_f64() / frame_duration.0.as_secs_f64()
    }

    pub fn get_frame_index(&self, frame_duration: &FrameDuration, time_value: &TimeValue) -> FrameIndex {
        FrameIndex(self.get_fractional_frame_index(frame_duration, time_value).floor() as usize)
    }

    pub fn get_time_value(&self) -> &TimeValue {
        &self.0
    }
}

