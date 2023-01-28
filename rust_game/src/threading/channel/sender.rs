use std::sync::mpsc;
use crate::threading::channel::SendMetaData;
use crate::threading::eventhandling;
use crate::threading::eventhandling::EventHandlerTrait;
use crate::threading::eventhandling::EventOrStopThread::{Event, StopThread};

pub type SendError<T> = mpsc::SendError<(SendMetaData, T)>;

pub struct Sender<T> {
    sender: mpsc::Sender<(SendMetaData, T)>
}

impl<T> Sender<T> {

    pub fn new(sender: mpsc::Sender<(SendMetaData, T)>) -> Self {
        return Self{
            sender
        }
    }

    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        return self.sender.send((SendMetaData::new(), value));
    }

}

impl<T> eventhandling::Sender<T> {

    pub fn send_event(&self, event: T) -> eventhandling::SendResult<T> {
        return self.send(Event(event));
    }

    pub fn send_stop_thread(&self) -> eventhandling::SendResult<T> {
        return self.send(StopThread);
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self { sender: self.sender.clone() }
    }
}
