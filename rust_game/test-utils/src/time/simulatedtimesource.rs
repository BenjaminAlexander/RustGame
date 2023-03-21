use std::sync::{Arc, Mutex};
use commons::time::{TimeSource, TimeValue};

//static simulated_time: Mutex<TimeValue> = Mutex::new(TimeValue::from_seconds_since_epoch(0.0));

#[derive(Clone)]
pub struct SimulatedTimeSource {
    simulated_time: Arc<Mutex<TimeValue>>
}

impl SimulatedTimeSource {

    pub fn new() -> Self {
        return Self {
            simulated_time: Arc::new(Mutex::new(TimeValue::from_seconds_since_epoch(0.0)))
        };
    }

    pub fn set_simulated_time(&self, time_value: TimeValue) {
        *self.simulated_time.lock().unwrap() = time_value;
    }
}

impl Default for SimulatedTimeSource {
    fn default() -> Self {
        return Self::new();
    }
}

impl TimeSource for SimulatedTimeSource {
    fn now(&self) -> TimeValue {
        return *self.simulated_time.lock().unwrap();
    }
}