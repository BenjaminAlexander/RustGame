use crate::factory::FactoryTrait;
use crate::net::{
    UdpReadHandlerTrait,
};
use crate::threading;
use crate::threading::channel::Channel;
use crate::threading::eventhandling::{
    EventHandlerSender,
    EventHandlerTrait,
    EventOrStopThread,
};
use crate::threading::AsyncJoinCallBackTrait;
use std::io::Error;

pub struct ChannelThreadBuilder<Factory: FactoryTrait, T: Send + 'static> {
    factory: Factory,
    thread_builder: threading::ThreadBuilder,
    channel: Channel<Factory, T>,
}

impl<Factory: FactoryTrait, T: Send + 'static> ChannelThreadBuilder<Factory, T> {
    pub fn new(factory: Factory, thread_builder: threading::ThreadBuilder) -> Self {
        let channel = factory.new_channel();
        return Self {
            factory,
            channel,
            thread_builder,
        };
    }

    pub fn get_channel(&self) -> &Channel<Factory, T> {
        return &self.channel;
    }

    pub fn get_sender(&self) -> &Factory::Sender<T> {
        return self.get_channel().get_sender();
    }

    //TODO: maybe remove this guy
    pub fn clone_sender(&self) -> Factory::Sender<T> {
        return (*self.get_channel().get_sender()).clone();
    }

    pub fn take(self) -> (threading::ThreadBuilder, Channel<Factory, T>) {
        return (self.thread_builder, self.channel);
    }
}

impl<Factory: FactoryTrait, T: Send + 'static> ChannelThreadBuilder<Factory, EventOrStopThread<T>> {
    pub fn spawn_event_handler<U: EventHandlerTrait<Event = T>>(
        self,
        event_handler: U,
        join_call_back: impl AsyncJoinCallBackTrait<U::ThreadReturn>,
    ) -> Result<EventHandlerSender<Factory, T>, Error> {
        let factory = self.factory.clone();
        return factory.spawn_event_handler(self, event_handler, join_call_back);
    }
}
