use crate::threading::{Sender, Receiver};
use crate::threading::thread::{Thread, ThreadBuilder};
use std::sync::mpsc;
use std::sync::mpsc::{Sender as MpscSender, Receiver as MpscReceiver};
use std::marker::PhantomData;

pub trait ChannelThread<T> : Sized + Send + 'static
    where T: Send + 'static {

    fn build(self) -> (Sender<Self>, ThreadBuilder<T>) {

        let (sender, receiver): (MpscSender<Box<FnOnce(&mut Self) + Send + 'static>>, MpscReceiver<Box<FnOnce(&mut Self) + Send + 'static>>) = mpsc::channel();

        let thread = RawChannelThread{
            receiver: Receiver::<Self>::new(receiver),
            channelThread: self,
            uPhantom: PhantomData
        };

        let builder = thread.build();

        (Sender::<Self>::new(sender), builder)

    }

    fn run(self, receiver: Receiver<Self>) -> T;
}

struct RawChannelThread<T, U>
    where T: ChannelThread<U>,
          U: Send + 'static {

    receiver: Receiver<T>,
    channelThread: T,
    uPhantom: PhantomData<U>
}

impl<T, U> Thread<U> for RawChannelThread<T, U>
    where T: ChannelThread<U>,
          U: Send + 'static {

    fn run(self) -> U {
        let receiver = self.receiver;
        let channelThread = self.channelThread;
        channelThread.run(receiver)
    }
}