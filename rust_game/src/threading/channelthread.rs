use std::marker::PhantomData;
use crate::threading::{OldReceiver, OldSender, old_build_thread, old_channel};
use crate::threading::thread::{Thread, OldThreadBuilder};

pub trait ChannelThread<ThreadReturnType, MessageReturnType> : Sized + Send + 'static
    where ThreadReturnType: Send + 'static,
        MessageReturnType: 'static {

    fn build(self) -> (OldSender<Self, MessageReturnType>, OldThreadBuilder<RawChannelThread<Self, ThreadReturnType, MessageReturnType>>) {
        let (sender, receiver) = old_channel();
        self.build_from_channel(sender, receiver)
    }

    fn build_from_channel(self, sender: OldSender<Self, MessageReturnType>, receiver: OldReceiver<Self, MessageReturnType>) -> (OldSender<Self, MessageReturnType>, OldThreadBuilder<RawChannelThread<Self, ThreadReturnType, MessageReturnType>>) {

        let thread = RawChannelThread{
            receiver,
            channel_thread: self,
            u_phantom: PhantomData
        };

        (sender, old_build_thread(thread))
    }

    fn run(self, receiver: OldReceiver<Self, MessageReturnType>) -> ThreadReturnType;
}

pub struct RawChannelThread<ChannelThreadType, ThreadReturnType, MessageReturnType>
    where ChannelThreadType: ChannelThread<ThreadReturnType, MessageReturnType>,
          ThreadReturnType: Send + 'static,
          MessageReturnType: 'static {

    receiver: OldReceiver<ChannelThreadType, MessageReturnType>,
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