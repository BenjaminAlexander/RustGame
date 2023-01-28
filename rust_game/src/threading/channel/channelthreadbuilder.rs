use crate::threading::channel::{Channel, Sender};
use crate::threading::eventhandling::{EventHandlerTrait, EventOrStopThread, JoinHandle, EventHandlerThread};
use crate::threading::listener::{ListenerState, ListenerTrait};
use crate::threading::threadbuilder::ThreadBuilder;

//TODO: rename?
pub struct ChannelThreadBuilder<T: Send + 'static> {
    thread_builder: ThreadBuilder,
    channel: Channel<T>
}

impl<T: Send + 'static> ChannelThreadBuilder<T> {

    pub fn new(thread_builder: ThreadBuilder) -> Self {
        return Self {
            thread_builder,
            channel: Channel::new()
        };
    }

    pub fn take(self) -> (ThreadBuilder, Channel<T>) {
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

impl<T: Send + 'static> ChannelThreadBuilder<EventOrStopThread<T>> {

    pub fn spawn_event_handler<U: EventHandlerTrait<Event=T>>(self, event_handler: U) -> std::io::Result<JoinHandle<U::Event, U::ThreadReturn>> {

        let (thread_builder, channel) = self.take();

        let (sender, receiver) = channel.take();

        let thread = EventHandlerThread::new(
            receiver,
            event_handler
        );

        let join_handle = thread_builder.spawn_thread(thread)?;

        return Result::Ok(JoinHandle {
            sender,
            join_handle
        });
    }

    pub fn spawn_listener<U: ListenerTrait<Event=T>>(self, listener: U) -> std::io::Result<JoinHandle<U::Event, U::ThreadReturn>> {
        return self.spawn_event_handler(ListenerState::ReadyToListen(listener));
    }
}