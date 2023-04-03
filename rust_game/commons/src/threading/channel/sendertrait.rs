use std::sync::mpsc;
use crate::threading::channel::SendMetaData;
use crate::threading::eventhandling::{EventOrStopThread, EventSenderTrait, SendResult};
use crate::threading::eventhandling::EventOrStopThread::{Event, StopThread};

pub type SendError<T> = mpsc::SendError<(SendMetaData, T)>;

pub trait SenderTrait<T>: Clone + Send {
    fn send(&self, value: T) -> Result<(), SendError<T>>;
}

impl<T, U: SenderTrait<EventOrStopThread<T>>> EventSenderTrait<T> for U {
    fn send_event(&self, event: T) -> SendResult<T> {
        return self.send(Event(event));
    }

    fn send_stop_thread(&self) -> SendResult<T> {
        return self.send(StopThread);
    }
}
