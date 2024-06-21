use commons::time::TimeValue;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct SimulatedTimeSource {
    simulated_time: Arc<Mutex<TimeValue>>,
}

impl SimulatedTimeSource {
    pub fn new() -> Self {
        return Self {
            simulated_time: Arc::new(Mutex::new(TimeValue::from_secs_f64(0.0))),
        };
    }

    pub fn set_simulated_time(&self, time_value: TimeValue) {
        *self.simulated_time.lock().unwrap() = time_value;
    }

    pub fn now(&self) -> TimeValue {
        return *self.simulated_time.lock().unwrap();
    }
}
