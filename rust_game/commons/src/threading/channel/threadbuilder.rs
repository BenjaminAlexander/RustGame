use crate::factory::FactoryTrait;
use crate::threading::channel::Channel;
use crate::threading::eventhandling::{EventHandlerTrait, EventOrStopThread, EventHandlerThread};
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
            channel: Channel::new(&thread_builder.get_factory()),
            thread_builder
        };
    }

    pub fn take(self) -> (threading::ThreadBuilder<Factory>, Channel<Factory, T>) {
        return (self.thread_builder, self.channel);
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

        let (thread_builder, channel) = self.take();

        let (sender, receiver) = channel.take();

        let thread = EventHandlerThread::new(
            thread_builder.get_factory().clone(),
            receiver,
            event_handler
        );

        thread_builder.spawn_thread(thread, join_call_back)?;

        return Ok(sender);
    }

    pub fn spawn_listener<U: ListenerTrait<Event=T>>(self, listener: U, join_call_back: impl FnOnce(AsyncJoin<Factory, U::ThreadReturn>) + Send + 'static) -> std::io::Result<eventhandling::Sender<Factory, T>> {
        let event_handler = ListenerState::new(self.thread_builder.get_factory().clone(), listener);
        return self.spawn_event_handler(event_handler, join_call_back);
    }
}