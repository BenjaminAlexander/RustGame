use std::sync::mpsc;
use commons::factory::FactoryTrait;
use commons::threading::{AsyncJoin, channel, ThreadBuilder};
use commons::threading::channel::{Channel, SendMetaData};
use commons::threading::eventhandling::{EventHandlerTrait, EventOrStopThread, Sender};
use commons::time::TimeValue;
use crate::singlethreaded::eventhandling::EventHandlerHolder;
use crate::singlethreaded::{SingleThreadedSender, TimeQueue};
use crate::time::SimulatedTimeSource;

#[derive(Clone)]
pub struct SingleThreadedFactory {
    simulated_time_source: SimulatedTimeSource,
    time_queue: TimeQueue
}

impl SingleThreadedFactory {

    pub fn new() -> Self {

        let simulated_time_source = SimulatedTimeSource::new();
        let time_queue = TimeQueue::new(simulated_time_source.clone());

        return Self {
            simulated_time_source,
            time_queue
        }
    }

    pub fn get_simulated_time_source(&self) -> &SimulatedTimeSource {
        return &self.simulated_time_source;
    }

    pub fn get_time_queue(&self) -> &TimeQueue {
        return &self.time_queue;
    }
}

impl FactoryTrait for SingleThreadedFactory {
    type Sender<T: Send> = SingleThreadedSender<T>;

    fn now(&self) -> TimeValue {
        return self.simulated_time_source.now();
    }

    fn new_sender<T: Send>(&self, sender: mpsc::Sender<(SendMetaData, T)>) -> Self::Sender<T> {
        return SingleThreadedSender::new(channel::RealSender::new(self.clone(), sender));
    }

    fn spawn_event_handler<T: Send, U: EventHandlerTrait<Event=T>>(&self, thread_builder: ThreadBuilder<Self>, channel: Channel<Self, EventOrStopThread<T>>, event_handler: U, join_call_back: impl FnOnce(AsyncJoin<Self, U::ThreadReturn>) + Send + 'static) -> std::io::Result<Sender<Self, T>> {
        let (sender, receiver) = channel.take();

        //TODO: implement

        return Ok(sender);
    }
}