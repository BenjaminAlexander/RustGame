use std::sync::mpsc::{Sender as MpscSender, SendError as MpscSendError};
use core::fmt::Debug;
use serde::export::Formatter;
use core::fmt;

pub struct Sender<T> {
    sender: MpscSender<Box<dyn FnOnce(&mut T) + Send + 'static>>
}

impl<T> Sender<T> {

    pub fn new(sender: MpscSender<Box<dyn FnOnce(&mut T) + Send + 'static>>) -> Self {
        Sender{sender}
    }

    //TODO: Make this a Custom ResultType
    pub fn send<U>(&self, u: U) -> Result<(), SendError<T>>
        where U: FnOnce(&mut T) + Send + 'static {
        match self.sender.send(Box::new(u)) {
            Ok(()) => {
                Ok(())
            }
            Err(error) => {
                Err(SendError(error))
            }
        }
    }
}

impl<T> Clone for Sender<T> {

    fn clone(&self) -> Self {
        Self {sender: self.sender.clone()}
    }
}

pub struct SendError<T>(pub MpscSendError<Box<dyn FnOnce(&mut T) + Send>>);

impl<T> Debug for SendError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}