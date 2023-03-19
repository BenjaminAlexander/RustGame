use std::sync::Mutex;
use commons::time::{TimeSource, TimeValue};

static simulated_time: Mutex<TimeValue> = Mutex::new(TimeValue::from_seconds_since_epoch(0.0));

pub struct SimulatedTimeProvider;

impl SimulatedTimeProvider {
    pub fn set_simulated_time(time_value: &TimeValue) {
        let mut simulated_time_mutex_guard = simulated_time.lock().unwrap();
        *simulated_time_mutex_guard = *time_value;
    }
}

impl TimeSource for SimulatedTimeProvider {
    fn now() -> TimeValue {
        return *simulated_time.lock().unwrap();
    }
}