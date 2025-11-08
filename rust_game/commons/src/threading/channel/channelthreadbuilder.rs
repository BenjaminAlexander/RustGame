use crate::threading::channel::{
    Channel,
    Sender,
};

pub struct ChannelThreadBuilder<T: Send + 'static> {
    channel: Channel<T>,
}

impl<T: Send + 'static> ChannelThreadBuilder<T> {
    pub fn new(channel: Channel<T>) -> Self {
        return Self { channel };
    }

    pub fn get_channel(&self) -> &Channel<T> {
        return &self.channel;
    }

    pub fn get_sender(&self) -> &Sender<T> {
        return self.get_channel().get_sender();
    }

    pub fn take(self) -> Channel<T> {
        return self.channel;
    }
}
