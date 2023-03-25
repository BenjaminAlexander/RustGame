use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::factory::FactoryTrait;
use crate::threading::channel::{Channel, RealSender, SendMetaData};
use crate::threading::eventhandling::{EventHandlerThread, EventHandlerTrait, EventOrStopThread, Sender};
use crate::threading::{AsyncJoin, ThreadBuilder};
use crate::time::TimeValue;

#[derive(Clone, Copy)]
pub struct RealFactory {

}

impl RealFactory {
    pub fn new() -> Self {
        return Self {};
    }
}

impl FactoryTrait for RealFactory {
    type Sender<T: Send> = RealSender<Self, T>;

    fn now(&self) -> TimeValue {
        return TimeValue::from_seconds_since_epoch(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64());
    }

    fn new_sender<T: Send>(&self, sender: mpsc::Sender<(SendMetaData, T)>) -> Self::Sender<T> {
        return RealSender::new(self.clone(), sender);
    }

    fn spawn_event_handler<T: Send, U: EventHandlerTrait<Event=T>>(&self, thread_builder: ThreadBuilder<Self>, channel: Channel<Self, EventOrStopThread<T>>, event_handler: U, join_call_back: impl FnOnce(AsyncJoin<Self, U::ThreadReturn>) + Send + 'static) -> std::io::Result<Sender<Self, T>> {
        let (sender, receiver) = channel.take();

        let thread = EventHandlerThread::new(
            thread_builder.get_factory().clone(),
            receiver,
            event_handler
        );

        thread_builder.spawn_thread(thread, join_call_back)?;

        return Ok(sender);
    }
}