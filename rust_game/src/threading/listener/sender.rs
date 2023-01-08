use crate::threading::eventhandling;
use crate::threading::listener::eventhandler::ListenerState;
use crate::threading::listener::ListenerTrait;

pub struct SendError<T: ListenerTrait> {
    event_send_error: eventhandling::SendError<ListenerState<T>>
}

pub type SendResult<T> = Result<(), SendError<T>>;

pub struct Sender<T: ListenerTrait> {
    pub(super) sender: eventhandling::Sender<ListenerState<T>>
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