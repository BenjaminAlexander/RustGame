use crate::threading;
use crate::threading::channel::{
    Channel,
    Sender,
};

pub struct ChannelThreadBuilder<T: Send + 'static> {
    thread_builder: threading::ThreadBuilder,
    channel: Channel<T>,
}

impl<T: Send + 'static> ChannelThreadBuilder<T> {
    pub fn new(channel: Channel<T>, thread_builder: threading::ThreadBuilder) -> Self {
        return Self {
            channel,
            thread_builder,
        };
    }

    pub fn get_channel(&self) -> &Channel<T> {
        return &self.channel;
    }

    pub fn get_sender(&self) -> &Sender<T> {
        return self.get_channel().get_sender();
    }

    //TODO: maybe remove this guy
    pub fn clone_sender(&self) -> Sender<T> {
        return (*self.get_channel().get_sender()).clone();
    }

    pub fn take(self) -> (threading::ThreadBuilder, Channel<T>) {
        return (self.thread_builder, self.channel);
    }
}
