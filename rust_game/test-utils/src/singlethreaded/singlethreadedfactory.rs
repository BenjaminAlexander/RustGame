use std::sync::mpsc;
use commons::factory::FactoryTrait;
use commons::threading::channel;
use commons::threading::channel::SendMetaData;
use commons::time::TimeValue;
use crate::singlethreaded::channel::SingleThreadedSender;
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
    type Sender<T: Send> = SingleThreadedSender<T>;

    fn now(&self) -> TimeValue {
        return self.simulated_time_source.now();
    }

    fn new_sender<T: Send>(&self, sender: mpsc::Sender<(SendMetaData, T)>) -> Self::Sender<T> {
        return SingleThreadedSender::new(channel::Sender::new(sender));
    }
}