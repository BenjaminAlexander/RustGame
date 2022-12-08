use std::marker::PhantomData;
use crate::threading::{Receiver, Sender, channel};
use crate::threading::thread::{Thread, ThreadBuilder};

pub trait ChannelThread<ThreadReturnType> : Sized + Send + 'static
    where ThreadReturnType: Send + 'static {

    fn build(self) -> (Sender<Self>, ThreadBuilder<ThreadReturnType>) {
        let (sender, receiver) = channel();
        self.build_from_channel(sender, receiver)
    }

    fn build_from_channel(self, sender: Sender<Self>, receiver: Receiver<Self>) -> (Sender<Self>, ThreadBuilder<ThreadReturnType>) {

        let thread = RawChannelThread{
            receiver,
            channel_thread: self,
            u_phantom: PhantomData
        };

        let builder = thread.build();

        (sender, builder)
    }

    fn run(self, receiver: Receiver<Self>) -> ThreadReturnType;
}

struct RawChannelThread<ChannelThreadType, ThreadReturnType>
    where ChannelThreadType: ChannelThread<ThreadReturnType>,
          ThreadReturnType: Send + 'static {

    receiver: Receiver<ChannelThreadType>,
    channel_thread: ChannelThreadType,
    u_phantom: PhantomData<ThreadReturnType>
}

impl<ChannelThreadType, ThreadReturnType> Thread<ThreadReturnType> for RawChannelThread<ChannelThreadType, ThreadReturnType>
    where ChannelThreadType: ChannelThread<ThreadReturnType>,
          ThreadReturnType: Send + 'static {

    fn run(self) -> ThreadReturnType {
        let receiver = self.receiver;
        let channel_thread = self.channel_thread;
        channel_thread.run(receiver)
    }
}