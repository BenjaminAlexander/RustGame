use std::sync::mpsc;
use crate::threading::channel::{Receiver, Sender, SendMetaData};

pub struct Channel<T: Send + 'static> {
    sender: Sender<T>,
    receiver: Receiver<T>
}

impl<T: Send + 'static> Channel<T> {

    pub fn new() -> Self {
        let (sender, receiver): (mpsc::Sender<(SendMetaData, T)>, mpsc::Receiver<(SendMetaData, T)>) = mpsc::channel();

        return Self {
            sender : Sender::new(sender),
            receiver: Receiver::new(receiver)
        };
    }

    pub fn get_sender(&self) -> &Sender<T> {
        return &self.sender;
    }

    pub fn get_receiver(&self) -> &Receiver<T> {
        return &self.receiver;
    }

    pub fn take(self) -> (Sender<T>, Receiver<T>) {
        return (self.sender, self.receiver);
    }
}