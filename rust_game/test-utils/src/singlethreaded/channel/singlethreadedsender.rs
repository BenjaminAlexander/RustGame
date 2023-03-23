use commons::factory::FactoryTrait;
use commons::threading::channel;
use commons::threading::channel::{SendError, SenderTrait};

pub struct SingleThreadedSender<T: Send> {
    sender: channel::Sender<T>
}

impl<T: Send> SingleThreadedSender<T> {
    pub fn new(sender: channel::Sender<T>) -> Self {
        return Self {
            sender
        };
    }
}

impl<T: Send> SenderTrait<T> for SingleThreadedSender<T> {
    fn send(&self, factory: &impl FactoryTrait, value: T) -> Result<(), SendError<T>> {
        return self.sender.send(factory, value);
    }
}

impl<T: Send> Clone for SingleThreadedSender<T> {
    fn clone(&self) -> Self {
        return Self {
            sender: self.sender.clone()
        };
    }
}