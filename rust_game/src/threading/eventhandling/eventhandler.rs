use std::ops::ControlFlow;
use crate::threading::{message_channel, Thread};
use crate::threading::eventhandling::{ChannelEvent, EventSender, WaitOrTryForNextEvent};
use crate::threading::eventhandling::thread::Thread as EventThread;
use crate::threading::eventhandling::threadbuilder::ThreadBuilder;

pub type EventHandlerResult<T: EventHandlerTrait> = ControlFlow<T::ThreadReturnType, WaitOrTryForNextEvent<T>>;

pub trait EventHandlerTrait: Send + Sized + 'static {
    type Event: Send + 'static;
    type ThreadReturnType: Send + 'static;

    fn build_thread(self) -> ThreadBuilder<Self> {
        let (sender, receiver) = message_channel();
        return ThreadBuilder {
            sender: EventSender { sender },
            builder: EventThread {
                receiver,
                message_handler: self
            }.build()
        };
    }

    fn on_event(self, event: ChannelEvent<Self>) -> EventHandlerResult<Self>;

    fn on_stop(self) -> Self::ThreadReturnType;
}