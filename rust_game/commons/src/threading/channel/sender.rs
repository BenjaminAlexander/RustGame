use std::sync::mpsc;
use crate::factory::FactoryTrait;
use crate::threading::channel::SendMetaData;
use crate::threading::eventhandling;
use crate::threading::eventhandling::EventOrStopThread::{Event, StopThread};

pub type SendError<T> = mpsc::SendError<(SendMetaData, T)>;

pub struct Sender<Factory: FactoryTrait, T> {
    sender: mpsc::Sender<(SendMetaData, T)>,
    factory: Factory
}

impl<Factory: FactoryTrait, T> Sender<Factory, T> {

    pub fn new(factory: Factory, sender: mpsc::Sender<(SendMetaData, T)>) -> Self {
        return Self{
            sender,
            factory
        }
    }

    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        return self.sender.send((SendMetaData::new(&self.factory), value));
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

impl<Factory: FactoryTrait, T> Clone for Sender<Factory, T> {
    fn clone(&self) -> Self {
        return Self {
            sender: self.sender.clone(),
            factory: self.factory.clone(),
        };
    }
}
