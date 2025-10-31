use std::ops::Add;

use commons::time::{
    TimeDuration,
    TimeValue,
};
use serde::{
    Deserialize,
    Serialize,
};

/// A TimeMessage represents a tick of the game clock as close to a Frame's time of occurance as possible.
/// These messages are used to propogate the occurance of a new frame from the clock throughout the system.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct TimeMessage {

    /// The time of occurance for frame index 0.  On clients, this time can float around to slave the client's clock to the server's clock.
    start: TimeValue,

    /// The time duration between each frame
    frame_duration: TimeDuration,

    /// The actual time this message was create by the clock
    actual_time: TimeValue,
}

impl TimeMessage {
    pub(super) fn new(
        start: TimeValue,
        frame_duration: TimeDuration,
        actual_time: TimeValue,
    ) -> Self {
        TimeMessage {
            start,
            frame_duration,
            actual_time,
        }
    }

    // pub fn get_actual_time(&self) -> &TimeValue {
    //     &self.actual_time
    // }

    pub fn is_after(&self, other: &TimeMessage) -> bool {
        self.actual_time.is_after(&other.actual_time)
    }

    pub fn get_step_from_actual_time(&self, actual_time: TimeValue) -> f64 {
        let duration_since_start = actual_time.duration_since(&self.start);
        return duration_since_start.as_secs_f64() / self.frame_duration.as_secs_f64();
    }

    pub fn get_step(&self) -> usize {
        return self.get_step_from_actual_time(self.actual_time).round() as usize;
    }

    pub fn get_scheduled_time(&self) -> TimeValue {
        self.start
            .add(&self.frame_duration.mul_f64(self.get_step() as f64))
    }

    pub fn get_lateness(&self) -> TimeDuration {
        self.get_scheduled_time().duration_since(&self.actual_time)
    }

    pub fn get_duration_since_start(&self, time_value: TimeValue) -> TimeDuration {
        return time_value.duration_since(&self.start);
    }

    pub fn get_step_duration(&self) -> TimeDuration {
        return self.frame_duration;
    }
}
