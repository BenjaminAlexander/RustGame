use std::marker::PhantomData;
use crate::threading::{Receiver, Sender, channel};
use crate::threading::thread::{Thread, ThreadBuilder};

pub trait ChannelThread<T> : Sized + Send + 'static
    where T: Send + 'static {

    fn build(self) -> (Sender<Self>, ThreadBuilder<T>) {
        let (sender, receiver) = channel();
        self.build_from_channel(sender, receiver)
    }

    fn build_from_channel(self, sender: Sender<Self>, receiver: Receiver<Self>) -> (Sender<Self>, ThreadBuilder<T>) {

        let thread = RawChannelThread{
            receiver,
            channel_thread: self,
            u_phantom: PhantomData
        };

        let builder = thread.build();

        (sender, builder)
    }

    fn run(self, receiver: Receiver<Self>) -> T;
}

struct RawChannelThread<T, U>
    where T: ChannelThread<U>,
          U: Send + 'static {

    receiver: Receiver<T>,
    channel_thread: T,
    u_phantom: PhantomData<U>
}

impl<T, U> Thread<U> for RawChannelThread<T, U>
    where T: ChannelThread<U>,
          U: Send + 'static {

    fn run(self) -> U {
        let receiver = self.receiver;
        let channel_thread = self.channel_thread;
        channel_thread.run(receiver)
    }
}