use std::io;
use std::sync::mpsc;
use crate::threading::channel::{Channel, SenderTrait, SendMetaData};
use crate::threading::{AsyncJoin, eventhandling, ThreadBuilder};
use crate::threading::eventhandling::{EventHandlerTrait, EventOrStopThread};
use crate::time::TimeValue;

pub trait FactoryTrait: Clone + Send + 'static {
    type Sender<T: Send>: SenderTrait<T>;

    fn now(&self) -> TimeValue;

    fn new_sender<T: Send>(&self, sender: mpsc::Sender<(SendMetaData, T)>) -> Self::Sender<T>;

    fn spawn_event_handler<T: Send, U: EventHandlerTrait<Event=T>>(
        &self,
        thread_builder: ThreadBuilder<Self>,
        channel: Channel<Self, EventOrStopThread<T>>,
        event_handler: U,
        join_call_back: impl FnOnce(AsyncJoin<Self, U::ThreadReturn>) + Send + 'static
    ) -> io::Result<eventhandling::Sender<Self, T>>;
}