use std::{
    sync::{
        Arc,
        Mutex,
    },
    time::SystemTime,
};

use super::TimeValue;

#[derive(Clone)]
pub struct TimeSource {
    //TODO: conditionally compile this field
    simulated_time_source: Option<SimulatedTimeSource>,
}

impl TimeSource {
    pub fn new() -> Self {
        return Self {
            simulated_time_source: None,
        };
    }

    pub fn new_simulated_time_source() -> (Self, SimulatedTimeSource) {
        let simulated_time_source = SimulatedTimeSource::new();

        let time_source = Self {
            simulated_time_source: Some(simulated_time_source.clone()),
        };

        return (time_source, simulated_time_source);
    }

    pub fn now(&self) -> TimeValue {
        if let Some(simulated_time_source) = &self.simulated_time_source {
            return simulated_time_source.now();
        }

        return TimeValue::from_system_time(&SystemTime::now()).unwrap();
    }
}

#[derive(Clone)]
pub struct SimulatedTimeSource {
    simulated_time: Arc<Mutex<TimeValue>>,
}

impl SimulatedTimeSource {
    fn new() -> Self {
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
