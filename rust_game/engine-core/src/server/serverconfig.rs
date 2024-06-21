use commons::time::TimeDuration;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct ServerConfig {
    step_duration: TimeDuration,
}

impl ServerConfig {
    pub fn new(step_duration: TimeDuration) -> Self {
        return Self { step_duration };
    }

    pub fn get_step_duration(&self) -> TimeDuration {
        return self.step_duration;
    }
}
