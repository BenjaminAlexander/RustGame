use commons::time::TimeValue;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    game_time::FrameDuration,
    FrameIndex,
};

/// The time of occurance for frame index 0.  On clients, this time can float around to slave the client's clock to the server's clock.
#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct StartTime(TimeValue);

impl StartTime {
    pub fn new(start_time: TimeValue) -> Self {
        Self(start_time)
    }

    /// Calculates the time of occurance of a particular [`FrameIndex`]
    pub fn get_frame_time_of_occurence(
        &self,
        frame_duration: &FrameDuration,
        frame_index: &FrameIndex,
    ) -> TimeValue {
        self.0 + frame_duration.duration_from_start(frame_index)
    }

    /// Calculates a floating point number that represents the fractional [`FrameIndex`] of a [`TimeValue`]
    pub fn get_fractional_frame_index(
        &self,
        frame_duration: &FrameDuration,
        time_value: &TimeValue,
    ) -> f64 {
        let duration_since_start = time_value.duration_since(&self.0);
        duration_since_start.as_secs_f64() / frame_duration.get_frame_duration().as_secs_f64()
    }

    /// Calculates the [`FrameIndex`] most recently prior to a [`TimeValue`]
    pub fn get_frame_index(
        &self,
        frame_duration: &FrameDuration,
        time_value: &TimeValue,
    ) -> FrameIndex {
        FrameIndex::from(
            self.get_fractional_frame_index(frame_duration, time_value)
                .floor() as usize,
        )
    }

    pub fn get_time_value(&self) -> &TimeValue {
        &self.0
    }
}
