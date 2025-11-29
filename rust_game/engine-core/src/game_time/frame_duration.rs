use commons::time::TimeDuration;
use serde::{
    Deserialize,
    Serialize,
};

use crate::FrameIndex;

/// The [`TimeDuration`] between frames
#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct FrameDuration(TimeDuration);

impl FrameDuration {
    pub fn new(frame_duration: TimeDuration) -> Self {
        Self(frame_duration)
    }

    pub fn get_frame_duration(&self) -> &TimeDuration {
        &self.0
    }

    pub fn duration_from_start(&self, frame_index: &FrameIndex) -> TimeDuration {
        self.0.mul_f64(frame_index.usize() as f64)
    }

    pub fn to_frame_count(&self, time_duration: &TimeDuration) -> f64 {
        time_duration.as_secs_f64() / self.0.as_secs_f64()
    }
}
