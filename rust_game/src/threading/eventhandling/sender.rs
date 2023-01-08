use crate::threading::eventhandling::eventhandlertrait::EventHandlerTrait;
use crate::threading::{ValueSender, ValueSendError};
use crate::threading::eventhandling::EventOrStopThread;
use crate::threading::eventhandling::EventOrStopThread::{Event, StopThread};

pub type SendError<T> = ValueSendError<EventOrStopThread<T>>;

pub type SendResult<T> = Result<(), SendError<T>>;

pub struct SentEventHolder<T: EventHandlerTrait> {
    pub(super) event: T::Event
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