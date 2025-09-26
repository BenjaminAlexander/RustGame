use crate::factory::FactoryTrait;

pub struct Channel<Factory: FactoryTrait, T: Send + 'static> {
    sender: Factory::Sender<T>,
    receiver: Factory::Receiver<T>,
}

impl<Factory: FactoryTrait, T: Send + 'static> Channel<Factory, T> {
    pub fn new(sender: Factory::Sender<T>, receiver: Factory::Receiver<T>) -> Self {
        return Self { sender, receiver };
    }

    pub fn get_sender(&self) -> &Factory::Sender<T> {
        return &self.sender;
    }

    pub fn take(self) -> (Factory::Sender<T>, Factory::Receiver<T>) {
        return (self.sender, self.receiver);
    }
}
