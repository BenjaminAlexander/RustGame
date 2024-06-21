use crate::threading::channel::SendMetaData;
use crate::threading::eventhandling::EventOrStopThread::{Event, StopThread};
use crate::threading::eventhandling::{
    EventHandlerSendResult, EventOrStopThread, EventSenderTrait,
};
use std::sync::mpsc;

pub type SendError<T> = mpsc::SendError<(SendMetaData, T)>;

pub trait SenderTrait<T>: Clone + Send {
    fn send(&self, value: T) -> Result<(), SendError<T>>;
}

impl<T, U: SenderTrait<EventOrStopThread<T>>> EventSenderTrait<T> for U {
    fn send_event(&self, event: T) -> EventHandlerSendResult<T> {
        return self.send(Event(event));
    }

    fn send_stop_thread(&self) -> EventHandlerSendResult<T> {
        return self.send(StopThread);
    }
}
