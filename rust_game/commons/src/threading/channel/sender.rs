use std::sync::mpsc;
use crate::factory::FactoryTrait;
use crate::threading::channel::SendMetaData;
use crate::threading::eventhandling;
use crate::threading::eventhandling::EventOrStopThread::{Event, StopThread};

pub type SendError<T> = mpsc::SendError<(SendMetaData, T)>;

pub struct Sender<T> {
    sender: mpsc::Sender<(SendMetaData, T)>,

}

impl<T> Sender<T> {

    pub fn new(sender: mpsc::Sender<(SendMetaData, T)>) -> Self {
        return Self{
            sender
        }
    }

    pub fn send(&self, factory: &impl FactoryTrait, value: T) -> Result<(), SendError<T>> {
        return self.sender.send((SendMetaData::new(factory), value));
    }

}

impl<T> eventhandling::Sender<T> {

    pub fn send_event(&self, factory: &impl FactoryTrait, event: T) -> eventhandling::SendResult<T> {
        return self.send(factory, Event(event));
    }

    pub fn send_stop_thread(&self, factory: &impl FactoryTrait) -> eventhandling::SendResult<T> {
        return self.send(factory, StopThread);
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        return Self {
            sender: self.sender.clone()
        };
    }
}
