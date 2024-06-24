use crate::threading::eventhandling::EventOrStopThread::{
    Event,
    StopThread,
};
use crate::threading::eventhandling::{
    EventHandlerSendResult,
    EventOrStopThread,
    EventSenderTrait,
};

pub trait SenderTrait<T>: Clone + Send {
    fn send(&self, value: T) -> Result<(), T>;
}

impl<T, U: SenderTrait<EventOrStopThread<T>>> EventSenderTrait<T> for U {
    fn send_event(&self, event: T) -> EventHandlerSendResult<T> {
        return self.send(Event(event));
    }

    fn send_stop_thread(&self) -> EventHandlerSendResult<T> {
        return self.send(StopThread);
    }
}
