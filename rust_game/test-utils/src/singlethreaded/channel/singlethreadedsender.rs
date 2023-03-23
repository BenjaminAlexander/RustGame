use commons::factory::FactoryTrait;
use commons::threading::channel;
use commons::threading::channel::{SendError, SenderTrait};
use crate::singlethreaded::SingleThreadedFactory;

pub struct SingleThreadedSender<T: Send> {
    sender: channel::RealSender<SingleThreadedFactory, T>
}

impl<T: Send> SingleThreadedSender<T> {
    pub fn new(sender: channel::RealSender<SingleThreadedFactory, T>) -> Self {
        return Self {
            sender
        };
    }
}

impl<T: Send> SenderTrait<T> for SingleThreadedSender<T> {
    fn send(&self, value: T) -> Result<(), SendError<T>> {
        return self.sender.send(value);
    }
}

impl<T: Send> Clone for SingleThreadedSender<T> {
    fn clone(&self) -> Self {
        return Self {
            sender: self.sender.clone()
        };
    }
}