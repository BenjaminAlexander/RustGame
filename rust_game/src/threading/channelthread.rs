use std::marker::PhantomData;
use crate::threading::{Receiver, Sender, channel};
use crate::threading::thread::{Thread, ThreadBuilder};

pub trait ChannelThread<ThreadReturnType, MessageReturnType> : Sized + Send + 'static
    where ThreadReturnType: Send + 'static,
        MessageReturnType: 'static {

    fn build(self) -> (Sender<Self, MessageReturnType>, ThreadBuilder<RawChannelThread<Self, ThreadReturnType, MessageReturnType>>) {
        let (sender, receiver) = channel();
        self.build_from_channel(sender, receiver)
    }

    fn build_from_channel(self, sender: Sender<Self, MessageReturnType>, receiver: Receiver<Self, MessageReturnType>) -> (Sender<Self, MessageReturnType>, ThreadBuilder<RawChannelThread<Self, ThreadReturnType, MessageReturnType>>) {

        let thread = RawChannelThread{
            receiver,
            channel_thread: self,
            u_phantom: PhantomData
        };

        let builder = thread.build();

        (sender, builder)
    }

    fn run(self, receiver: Receiver<Self, MessageReturnType>) -> ThreadReturnType;
}

pub struct RawChannelThread<ChannelThreadType, ThreadReturnType, MessageReturnType>
    where ChannelThreadType: ChannelThread<ThreadReturnType, MessageReturnType>,
          ThreadReturnType: Send + 'static,
          MessageReturnType: 'static {

    receiver: Receiver<ChannelThreadType, MessageReturnType>,
    channel_thread: ChannelThreadType,
    u_phantom: PhantomData<ThreadReturnType>
}

impl<ChannelThreadType, ThreadReturnType, MessageReturnType> Thread for RawChannelThread<ChannelThreadType, ThreadReturnType, MessageReturnType>
    where ChannelThreadType: ChannelThread<ThreadReturnType, MessageReturnType>,
          ThreadReturnType: Send + 'static {

    type ReturnType = ThreadReturnType;

    fn run(self) -> ThreadReturnType {
        let receiver = self.receiver;
        let channel_thread = self.channel_thread;
        channel_thread.run(receiver)
    }
}