use commons::factory::FactoryTrait;
use commons::time::TimeValue;
use crate::time::SimulatedTimeSource;

#[derive(Clone)]
pub struct SingleThreadedFactory {
    simulated_time_source: SimulatedTimeSource
}

impl SingleThreadedFactory {

    pub fn new() -> Self {
        return Self {
            simulated_time_source: SimulatedTimeSource::new()
        }
    }

    pub fn get_simulated_time_source(&self) -> &SimulatedTimeSource {
        return &self.simulated_time_source;
    }
}

impl FactoryTrait for SingleThreadedFactory {
    fn now(&self) -> TimeValue {
        return self.simulated_time_source.now();
    }
}