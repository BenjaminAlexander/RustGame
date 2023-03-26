use crate::factory::FactoryTrait;
use crate::threading::channel::Receiver;

pub struct Channel<Factory: FactoryTrait, T: Send + 'static> {
    sender: Factory::Sender<T>,
    receiver: Receiver<T>
}

impl<Factory: FactoryTrait, T: Send + 'static> Channel<Factory, T> {

    pub fn new(sender: Factory::Sender<T>, receiver: Receiver<T>) -> Self {
        return Self {
            sender,
            receiver
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