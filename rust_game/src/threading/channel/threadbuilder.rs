use crate::threading::channel::{Channel, Sender};
use crate::threading::eventhandling::{EventHandlerTrait, EventOrStopThread, EventHandlerThread};
use crate::threading::listener::{ListenerState, ListenerTrait};
use crate::threading;
use crate::threading::{AsyncJoin, eventhandling};

pub struct ThreadBuilder<T: Send + 'static> {
    thread_builder: threading::ThreadBuilder,
    channel: Channel<T>
}

impl<T: Send + 'static> ThreadBuilder<T> {

    pub fn new(thread_builder: threading::ThreadBuilder) -> Self {
        return Self {
            thread_builder,
            channel: Channel::new()
        };
    }

    pub fn take(self) -> (threading::ThreadBuilder, Channel<T>) {
        return (self.thread_builder, self.channel);
    }

    pub fn get_channel(&self) -> &Channel<T> {
        return &self.channel;
    }

    pub fn get_sender(&self) -> &Sender<T> {
        return self.get_channel().get_sender();
    }

    pub fn clone_sender(&self) -> Sender<T> {
        return self.get_channel().get_sender().clone();
    }
}

impl<T: Send + 'static> ThreadBuilder<EventOrStopThread<T>> {

    pub fn spawn_event_handler<U: EventHandlerTrait<Event=T>>(self, event_handler: U, join_call_back: impl FnOnce(AsyncJoin<U::ThreadReturn>) + Send + 'static) -> std::io::Result<eventhandling::Sender<T>> {

        let (thread_builder, channel) = self.take();

        let (sender, receiver) = channel.take();

        let thread = EventHandlerThread::new(
            receiver,
            event_handler
        );

        thread_builder.spawn_thread(thread, join_call_back)?;

        return Result::Ok(sender);
    }

    pub fn spawn_listener<U: ListenerTrait<Event=T>>(self, listener: U, join_call_back: impl FnOnce(AsyncJoin<U::ThreadReturn>) + Send + 'static) -> std::io::Result<eventhandling::Sender<T>> {
        return self.spawn_event_handler(ListenerState::ReadyToListen(listener), join_call_back);
    }
}