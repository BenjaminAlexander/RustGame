use std::io::Error;
use crate::factory::FactoryTrait;
use crate::net::TcpReadHandlerTrait;
use crate::threading::channel::Channel;
use crate::threading::eventhandling::{EventHandlerTrait, EventOrStopThread};
use crate::threading::listener::{ListenerState, ListenerTrait};
use crate::threading;
use crate::threading::{AsyncJoinCallBackTrait, eventhandling};

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

    pub fn take(self) -> (threading::ThreadBuilder<Factory>, Channel<Factory, T>) {
        return (self.thread_builder, self.channel);
    }
}

impl<Factory: FactoryTrait, T: Send + 'static> ThreadBuilder<Factory, EventOrStopThread<T>> {

    pub fn spawn_event_handler<U: EventHandlerTrait<Event=T>>(self, event_handler: U, join_call_back: impl AsyncJoinCallBackTrait<Factory, U::ThreadReturn>) -> std::io::Result<eventhandling::Sender<Factory, T>> {
        let factory = self.thread_builder.get_factory().clone();
        return factory.spawn_event_handler(self.thread_builder, self.channel, event_handler, join_call_back);
    }

    //TODO: remove
    pub fn spawn_listener<U: ListenerTrait<Event=T>>(self, listener: U, join_call_back: impl AsyncJoinCallBackTrait<Factory, U::ThreadReturn>) -> std::io::Result<eventhandling::Sender<Factory, T>> {
        let event_handler = ListenerState::new(self.thread_builder.get_factory().clone(), listener);
        return self.spawn_event_handler(event_handler, join_call_back);
    }
}

impl<Factory: FactoryTrait> ThreadBuilder<Factory, EventOrStopThread<()>> {

    pub fn spawn_tcp_reader<T: TcpReadHandlerTrait>(self, tcp_reader: Factory::TcpReader, tcp_read_handler: T, join_call_back: impl AsyncJoinCallBackTrait<Factory, T>) -> Result<eventhandling::Sender<Factory, ()>, Error> {
        let factory = self.thread_builder.get_factory().clone();
        return factory.spawn_tcp_reader(self, tcp_reader, tcp_read_handler, join_call_back);
    }

}