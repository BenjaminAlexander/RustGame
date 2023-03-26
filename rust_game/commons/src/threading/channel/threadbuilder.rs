use crate::factory::FactoryTrait;
use crate::threading::channel::Channel;
use crate::threading::eventhandling::{EventHandlerTrait, EventOrStopThread};
use crate::threading::listener::{ListenerState, ListenerTrait};
use crate::threading;
use crate::threading::{AsyncJoin, eventhandling};

pub struct ThreadBuilder<Factory: FactoryTrait, T: Send + 'static> {
    thread_builder: threading::ThreadBuilder<Factory>,
    channel: Channel<Factory, T>
}

impl<Factory: FactoryTrait, T: Send + 'static> ThreadBuilder<Factory, T> {

    pub fn new(thread_builder: threading::ThreadBuilder<Factory>) -> Self {
        return Self {
            channel: thread_builder.get_factory().new_channel(),
            thread_builder
        };
    }

    pub fn get_channel(&self) -> &Channel<Factory, T> {
        return &self.channel;
    }

    pub fn get_sender(&self) -> &Factory::Sender<T> {
        return self.get_channel().get_sender();
    }

    pub fn clone_sender(&self) -> Factory::Sender<T> {
        return self.get_channel().get_sender().clone();
    }
}

impl<Factory: FactoryTrait, T: Send + 'static> ThreadBuilder<Factory, EventOrStopThread<T>> {

    pub fn spawn_event_handler<U: EventHandlerTrait<Event=T>>(self, event_handler: U, join_call_back: impl FnOnce(AsyncJoin<Factory, U::ThreadReturn>) + Send + 'static) -> std::io::Result<eventhandling::Sender<Factory, T>> {
        let factory = self.thread_builder.get_factory().clone();
        return factory.spawn_event_handler(self.thread_builder, self.channel, event_handler, join_call_back);
    }

    pub fn spawn_listener<U: ListenerTrait<Event=T>>(self, listener: U, join_call_back: impl FnOnce(AsyncJoin<Factory, U::ThreadReturn>) + Send + 'static) -> std::io::Result<eventhandling::Sender<Factory, T>> {
        let event_handler = ListenerState::new(self.thread_builder.get_factory().clone(), listener);
        return self.spawn_event_handler(event_handler, join_call_back);
    }
}