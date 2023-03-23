use std::sync::mpsc;
use crate::factory::FactoryTrait;
use crate::threading::channel::{Receiver, SendMetaData};

pub struct Channel<Factory: FactoryTrait, T: Send + 'static> {
    sender: Factory::Sender<T>,
    receiver: Receiver<T>
}

impl<Factory: FactoryTrait, T: Send + 'static> Channel<Factory, T> {

    pub fn new(factory: &Factory) -> Self {
        let (sender, receiver): (mpsc::Sender<(SendMetaData, T)>, mpsc::Receiver<(SendMetaData, T)>) = mpsc::channel();

        return Self {
            sender : factory.new_sender(sender),
            receiver: Receiver::new(receiver)
        };
    }

    pub fn get_sender(&self) -> &Factory::Sender<T> {
        return &self.sender;
    }

    pub fn get_receiver(&self) -> &Receiver<T> {
        return &self.receiver;
    }

    pub fn take(self) -> (Factory::Sender<T>, Receiver<T>) {
        return (self.sender, self.receiver);
    }
}