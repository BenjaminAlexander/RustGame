use commons::time::{TimeDuration, TimeValue};
use serde::{
    Deserialize,
    Serialize,
};

// TODO: rename file

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct FrameIndex(usize);

impl FrameIndex {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn next(&self) -> FrameIndex {
        Self(self.0 + 1) 
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

    //TODO: get FrameIndex
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct StartTime(TimeValue);

impl StartTime {
    fn new(start_time: TimeValue) -> Self {
        Self(start_time)
    }

}

