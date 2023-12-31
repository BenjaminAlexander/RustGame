use std::sync::mpsc;
use crate::factory::FactoryTrait;
use crate::threading::channel::{SendError, SenderTrait, SendMetaData};

pub struct RealSender<Factory: FactoryTrait, T: Send> {
    factory: Factory,
    sender: mpsc::Sender<(SendMetaData, T)>,
}

impl<Factory: FactoryTrait, T: Send> RealSender<Factory, T> {

    pub fn new(factory: Factory, sender: mpsc::Sender<(SendMetaData, T)>) -> Self {
        return Self {
            factory,
            sender
        };
    }
}

impl<Factory: FactoryTrait, T: Send> SenderTrait<T> for RealSender<Factory, T> {
    fn send(&self, value: T) -> Result<(), SendError<T>> {
        return self.sender.send((SendMetaData::new(&self.factory), value));
    }
}

impl<Factory: FactoryTrait, T: Send> Clone for RealSender<Factory, T> {
    fn clone(&self) -> Self {
        return Self {
            factory: self.factory.clone(),
            sender: self.sender.clone()
        };
    }
}
