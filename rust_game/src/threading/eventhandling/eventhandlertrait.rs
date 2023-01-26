use std::ops::ControlFlow;
use crate::threading;
use crate::threading::channel::{Channel, ReceiveMetaData};
use crate::threading::eventhandling::{ChannelEvent, WaitOrTryForNextEvent};
use crate::threading::eventhandling::thread::Thread;
use crate::threading::eventhandling::threadbuilder::ThreadBuilder;

pub type ChannelEventResult<T> = ControlFlow<<T as EventHandlerTrait>::ThreadReturn, WaitOrTryForNextEvent<T>>;

pub trait EventHandlerTrait: Send + Sized + 'static {
    type Event: Send + 'static;
    type ThreadReturn: Send + 'static;

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> ChannelEventResult<Self>;

    fn on_stop(self, receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn;
}

pub fn build_thread<T: EventHandlerTrait>(event_handler: T) -> ThreadBuilder<T> {
    let (sender, receiver) = Channel::new().take();
    return ThreadBuilder {
        sender: sender,
        builder: threading::old_build_thread(Thread {
            receiver,
            event_handler
        })
    };
}