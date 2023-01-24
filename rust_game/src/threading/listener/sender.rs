use crate::threading::eventhandling;
use crate::threading::eventhandling::EventSenderTrait;
use crate::threading::listener::eventhandler::ListenerState;
use crate::threading::listener::ListenerTrait;

#[derive(Debug)]
pub struct SendError<T> {
    event_send_error: eventhandling::SendError<T>
}

pub type SendResult<T> = Result<(), SendError<<T as ListenerTrait>::Event>>;

pub struct Sender<T: ListenerTrait> {
    pub(super) sender: eventhandling::Sender<T::Event>
}

impl<T: ListenerTrait> Sender<T> {

    pub fn send_event(&self, event: T::Event) -> SendResult<T> {
        match self.sender.send_event(event) {
            Ok(()) => Ok(()),
            Err(event_send_error) => Err(SendError { event_send_error })
        }
    }

    pub fn send_stop_thread(&self) -> SendResult<T> {
        match self.sender.send_stop_thread() {
            Ok(()) => Ok(()),
            Err(event_send_error) => Err(SendError { event_send_error })
        }
    }
}

impl<T: ListenerTrait> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {sender: self.sender.clone()}
    }
}