use std::ops::ControlFlow;
use crate::threading;
use crate::threading::message_channel;
use crate::threading::eventhandling::{ChannelEvent, Sender, WaitOrTryForNextEvent};
use crate::threading::eventhandling::thread::Thread;
use crate::threading::eventhandling::threadbuilder::ThreadBuilder;

pub type ChannelEventResult<T: EventHandlerTrait> = ControlFlow<T::ThreadReturn, WaitOrTryForNextEvent<T>>;

pub trait EventHandlerTrait: Send + Sized + 'static {
    type Event: Send + 'static;
    type ThreadReturn: Send + 'static;

    fn on_channel_event(self, channel_event: ChannelEvent<Self>) -> ChannelEventResult<Self>;

    fn on_stop(self) -> Self::ThreadReturn;
}

pub fn build_thread<T: EventHandlerTrait>(event_handler: T) -> ThreadBuilder<T> {
    let (sender, receiver) = message_channel();
    return ThreadBuilder {
        sender: Sender { sender },
        builder: threading::build_thread(Thread {
            receiver,
            event_handler
        })
    };
}