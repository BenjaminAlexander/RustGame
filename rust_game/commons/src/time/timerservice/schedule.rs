use crate::time::{
    TimeDuration,
    TimeValue,
};

#[derive(Debug)]
pub enum Schedule {
    Once(TimeValue),
    Repeating(TimeValue, TimeDuration),
}

impl Schedule {
    pub fn get_trigger_time(&self) -> &TimeValue {
        return match self {
            Schedule::Once(time_value) => time_value,
            Schedule::Repeating(time_value, _) => time_value,
        };
    }

    pub fn should_trigger(&self, now: &TimeValue) -> bool {
        let trigger_time = self.get_trigger_time();
        return now.is_after(trigger_time) || trigger_time == now;
    }
}
