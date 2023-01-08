use crate::threading::eventhandling::eventhandler::EventHandlerTrait;
use crate::threading::{ValueSender, ValueSendError};
use crate::threading::eventhandling::EventOrStopThread;
use crate::threading::eventhandling::EventOrStopThread::{Event, StopThread};

pub type EventSendError<T> = ValueSendError<EventOrStopThread<T>>;

pub type EventSendResult<T> = Result<(), EventSendError<T>>;

pub struct SentEventHolder<T: EventHandlerTrait> {
    pub(super) event: T::Event
}

pub struct EventSender<T: EventHandlerTrait> {
    pub(super) sender: ValueSender<EventOrStopThread<T>>
}

impl<T: EventHandlerTrait> EventSender<T>{

    pub fn send_event(&self, event: T::Event) -> EventSendResult<T> {
        return self.sender.send(Event(SentEventHolder { event }));
    }

    pub fn send_stop_thread(&self) -> EventSendResult<T> {
        return self.sender.send(StopThread);
    }
}