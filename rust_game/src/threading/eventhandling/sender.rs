use crate::threading;
use crate::threading::eventhandling::eventhandlertrait::EventHandlerTrait;
use crate::threading::{ValueSender};
use crate::threading::eventhandling::EventOrStopThread;
use crate::threading::eventhandling::EventOrStopThread::{Event, StopThread};

pub type SendError<T> = threading::SendError<EventOrStopThread<T>>;

pub type SendResult<T> = Result<(), SendError<T>>;

pub struct SentEventHolder<T> {
    pub(super) event: T
}

pub struct Sender<T: EventHandlerTrait> {
    pub(super) sender: ValueSender<EventOrStopThread<T>>
}

impl<T: EventHandlerTrait> Sender<T>{

    pub fn send_event(&self, event: T::Event) -> SendResult<T> {
        return self.sender.send(Event(SentEventHolder { event }));
    }

    pub fn send_stop_thread(&self) -> SendResult<T> {
        return self.sender.send(StopThread);
    }
}