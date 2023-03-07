use crate::time::{TimeDuration, TimeValue};

pub enum Schedule {
    Once(TimeValue),
    Repeating(TimeValue, TimeDuration)
}

impl Schedule {

    pub fn get_trigger_time(&self) -> &TimeValue {
        return match self {
            Schedule::Once(time_value) => time_value,
            Schedule::Repeating(time_value, _) => time_value
        }
    }

    pub fn should_trigger(&self) -> bool {
        let now = TimeValue::now();
        let trigger_time = self.get_trigger_time();
        return trigger_time.is_after(&now) || *trigger_time == now;
    }
}