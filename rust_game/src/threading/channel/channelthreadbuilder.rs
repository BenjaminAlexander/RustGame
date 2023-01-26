use crate::threading::channel::{Channel, Sender};
use crate::threading::threadbuilder::ThreadBuilder;

pub struct ChannelThreadBuilder<T: Send + 'static> {
    thread_builder: ThreadBuilder,
    channel: Channel<T>
}

impl<T: Send + 'static> ChannelThreadBuilder<T> {

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

pub trait ChannelThreadBuilderBuilder {

    fn build_channel_thread<T: Send + 'static>(self) -> ChannelThreadBuilder<T>;

}

impl ChannelThreadBuilderBuilder for ThreadBuilder {

    fn build_channel_thread<T: Send + 'static>(self) -> ChannelThreadBuilder<T> {

        return ChannelThreadBuilder {
            thread_builder: self,
            channel: Channel::new()
        };
    }

}