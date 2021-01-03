use serde::{Deserialize, Serialize};
use crate::gametime::{TimeValue, TimeDuration};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct TimeMessage {
    start: TimeValue,
    duration: TimeDuration,
    actual_time: TimeValue,
}

impl TimeMessage {

    pub(super) fn new(start: TimeValue,
                      duration: TimeDuration,
                      actual_time: TimeValue) -> Self {

        TimeMessage{start, duration, actual_time}
    }

    // pub fn get_actual_time(&self) -> &TimeValue {
    //     &self.actual_time
    // }

    pub fn is_after(&self, other: &TimeMessage) -> bool {
        self.actual_time.is_after(&other.actual_time)
    }

    pub fn get_sequence(&self) -> i64 {
        let duration_since_start = self.actual_time.duration_since(self.start);
        (duration_since_start / self.duration).round() as i64
    }

    pub fn get_scheduled_time(&self) -> TimeValue {
        self.start.add(self.duration * self.get_sequence())
    }

    pub fn get_lateness(&self) -> TimeDuration {
        self.get_scheduled_time().duration_since(self.actual_time)
    }
}