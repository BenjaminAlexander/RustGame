use std::sync::mpsc::{Sender as MpscSender};

pub struct Sender<T> {
    sender: MpscSender<Box<dyn FnOnce(&mut T) + Send + 'static>>
}

impl<T> Sender<T> {

    pub fn new(sender: MpscSender<Box<dyn FnOnce(&mut T) + Send + 'static>>) -> Self {
        Sender{sender}
    }

    pub fn send<U>(&self, u: U)
        where U: FnOnce(&mut T) + Send + 'static
    {
        self.sender.send(Box::new(u));
    }
}

impl<T> Clone for Sender<T> {

    fn clone(&self) -> Self {
        Self {sender: self.sender.clone()}
    }
}