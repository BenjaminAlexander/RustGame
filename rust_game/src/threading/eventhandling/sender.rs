use crate::threading::channel;
use crate::threading::eventhandling::eventhandlertrait::EventHandlerTrait;
use crate::threading::eventhandling::EventOrStopThread;
use crate::threading::eventhandling::EventOrStopThread::{Event, StopThread};

//TODO: use event type not EventHandlerTrait type
pub type SendError<T> = channel::SendError<EventOrStopThread<T>>;

pub type SendResult<T> = Result<(), SendError<T>>;

//TODO: remove this type
pub type Sender<T> = channel::Sender<EventOrStopThread<T>>;

pub trait EventSenderTrait<T> {

    fn send_event(&self, event: T) -> SendResult<T>;

    fn send_stop_thread(&self) -> SendResult<T>;
}

impl<T> EventSenderTrait<T> for Sender<T> {

    fn send_event(&self, event: T) -> SendResult<T> {
        return self.send(Event(event));
    }

    fn send_stop_thread(&self) -> SendResult<T> {
        return self.send(StopThread);
    }
}