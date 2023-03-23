use std::sync::mpsc;
use crate::factory::FactoryTrait;
use crate::threading::channel::{SendError, SenderTrait, SendMetaData};

//TODO: rename as RealSender
pub struct Sender<T: Send> {
    sender: mpsc::Sender<(SendMetaData, T)>,
}

impl<T: Send> Sender<T> {

    pub fn new(sender: mpsc::Sender<(SendMetaData, T)>) -> Self {
        return Self{
            sender
        }
    }
}

impl<T: Send> SenderTrait<T> for Sender<T> {
    fn send(&self, factory: &impl FactoryTrait, value: T) -> Result<(), SendError<T>> {
        return self.sender.send((SendMetaData::new(factory), value));
    }
}

impl<T: Send> Clone for Sender<T> {
    fn clone(&self) -> Self {
        return Self {
            sender: self.sender.clone()
        };
    }
}
