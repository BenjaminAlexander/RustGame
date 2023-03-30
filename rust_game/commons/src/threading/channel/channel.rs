use crate::factory::FactoryTrait;
use crate::threading::channel::Receiver;

//TODO: maybe refactor to just use SenderTrait instead of factory
pub struct Channel<Factory: FactoryTrait, T: Send + 'static> {
    sender: Factory::Sender<T>,
    receiver: Receiver<Factory, T>
}

impl<Factory: FactoryTrait, T: Send + 'static> Channel<Factory, T> {

    pub fn new(sender: Factory::Sender<T>, receiver: Receiver<Factory, T>) -> Self {
        return Self {
            sender,
            receiver
        };
    }

    pub fn get_sender(&self) -> &Factory::Sender<T> {
        return &self.sender;
    }

    pub fn get_receiver(&self) -> &Receiver<Factory, T> {
        return &self.receiver;
    }

    pub fn take(self) -> (Factory::Sender<T>, Receiver<Factory, T>) {
        return (self.sender, self.receiver);
    }
}