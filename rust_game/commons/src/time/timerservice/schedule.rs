use crate::time::{
    TimeDuration,
    TimeValue,
};

/// A Set of ways to schedule the invocation of timers
#[derive(Debug)]
pub enum Schedule {
    /// Invoke the timer once
    Once(TimeValue),

    /// Invoke the timer repeatedly, starting at the time, then repeatedly after the duration
    Repeating(TimeValue, TimeDuration),

    /// Never invoke the timer
    Never,
}

impl Schedule {
    pub(super) fn get_trigger_time(&self) -> Option<&TimeValue> {
        return match self {
            Schedule::Once(time_value) => Some(time_value),
            Schedule::Repeating(time_value, _) => Some(time_value),
            Schedule::Never => None,
        };
    }

    pub(super) fn should_trigger(&self, now: &TimeValue) -> bool {
        return match self.get_trigger_time() {
            Some(trigger_time) => now.is_after(trigger_time) || trigger_time == now,
            None => false,
        };
    }
}
