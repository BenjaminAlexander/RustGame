use serde::{Deserialize, Serialize};
use commons::time::{TimeValue, TimeDuration};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct TimeMessage {
    start: TimeValue,
    step_duration: TimeDuration,
    actual_time: TimeValue,
}

impl TimeMessage {

    pub(super) fn new(start: TimeValue,
                      step_duration: TimeDuration,
                      actual_time: TimeValue) -> Self {

        TimeMessage{start, step_duration, actual_time}
    }

    // pub fn get_actual_time(&self) -> &TimeValue {
    //     &self.actual_time
    // }

    pub fn is_after(&self, other: &TimeMessage) -> bool {
        self.actual_time.is_after(&other.actual_time)
    }

    pub fn get_step_from_actual_time(&self, actual_time: TimeValue) -> f64 {
        let duration_since_start = actual_time.duration_since(self.start);
        return duration_since_start / self.step_duration;
    }

    pub fn get_step(&self) -> usize {
        return self.get_step_from_actual_time(self.actual_time).round() as usize;
    }

    pub fn get_scheduled_time(&self) -> TimeValue {
        self.start.add(self.step_duration * self.get_step() as i64)
    }

    pub fn get_lateness(&self) -> TimeDuration {
        self.get_scheduled_time().duration_since(self.actual_time)
    }

    pub fn get_duration_since_start(&self, time_value: TimeValue) -> TimeDuration {
        return time_value.duration_since(self.start);
    }

    pub fn get_step_duration(&self) -> TimeDuration {
        return self.step_duration;
    }
}