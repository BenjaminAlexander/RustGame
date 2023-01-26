use crate::threading::channel;
use crate::threading::eventhandling::eventhandlertrait::EventHandlerTrait;
use crate::threading::eventhandling::{EventOrStopThread, Sender, SendResult};
use crate::threading::eventhandling::EventOrStopThread::{Event, StopThread};

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