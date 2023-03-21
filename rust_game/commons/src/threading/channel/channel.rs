use std::sync::mpsc;
use crate::factory::FactoryTrait;
use crate::threading::channel::{Receiver, Sender, SendMetaData};

pub struct Channel<Factory: FactoryTrait, T: Send + 'static> {
    sender: Sender<Factory, T>,
    receiver: Receiver<T>
}

impl<Factory: FactoryTrait, T: Send + 'static> Channel<Factory, T> {

    pub fn new(factory: Factory) -> Self {
        let (sender, receiver): (mpsc::Sender<(SendMetaData, T)>, mpsc::Receiver<(SendMetaData, T)>) = mpsc::channel();

        return Self {
            sender : Sender::new(factory, sender),
            receiver: Receiver::new(receiver)
        };
    }

    pub fn get_sender(&self) -> &Sender<Factory, T> {
        return &self.sender;
    }

    pub fn get_receiver(&self) -> &Receiver<T> {
        return &self.receiver;
    }

    pub fn take(self) -> (Sender<Factory, T>, Receiver<T>) {
        return (self.sender, self.receiver);
    }
}