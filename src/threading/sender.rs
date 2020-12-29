use std::sync::mpsc::{Sender as MpscSender, SendError};

pub struct Sender<T> {
    sender: MpscSender<Box<dyn FnOnce(&mut T) + Send + 'static>>
}

impl<T> Sender<T> {

    pub fn new(sender: MpscSender<Box<dyn FnOnce(&mut T) + Send + 'static>>) -> Self {
        Sender{sender}
    }

    //TODO: Make this a Custom ResultType
    pub fn send<U>(&self, u: U) -> Result<(), SendError<Box<dyn FnOnce(&mut T) + Send>>>
        where U: FnOnce(&mut T) + Send + 'static {
        self.sender.send(Box::new(u))
    }
}

impl<T> Clone for Sender<T> {

    fn clone(&self) -> Self {
        Self {sender: self.sender.clone()}
    }
}