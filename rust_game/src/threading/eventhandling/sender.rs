use crate::threading::channel;
use crate::threading::eventhandling::eventhandlertrait::EventHandlerTrait;
use crate::threading::eventhandling::EventOrStopThread;
use crate::threading::eventhandling::EventOrStopThread::{Event, StopThread};

pub type SendError<T> = channel::SendError<EventOrStopThread<T>>;

pub type SendResult<T> = Result<(), SendError<<T as EventHandlerTrait>::Event>>;

pub struct Sender<T: EventHandlerTrait> {
    pub(super) sender: channel::Sender<EventOrStopThread<T::Event>>
}

impl<T: EventHandlerTrait> Sender<T> {

    pub fn send_event(&self, event: T::Event) -> SendResult<T> {
        return self.sender.send(Event(event));
    }

    pub fn send_stop_thread(&self) -> SendResult<T> {
        return self.sender.send(StopThread);
    }
}


impl<T: EventHandlerTrait> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {sender: self.sender.clone()}
    }
}