use std::sync::mpsc::{Sender as MpscSender, SendError as MpscSendError};
use core::fmt::Debug;
use serde::export::Formatter;
use core::fmt;

pub struct Sender<T, U> {
    sender: MpscSender<Box<dyn FnOnce(&mut T) -> U + Send + 'static>>
}

impl<T, U> Sender<T, U> {

    pub fn new(sender: MpscSender<Box<dyn FnOnce(&mut T) -> U + Send + 'static>>) -> Self {
        Sender{sender}
    }

    //TODO: Make this a Custom ResultType
    pub fn send<V>(&self, v: V) -> Result<(), SendError<T, U>>
        where V: FnOnce(&mut T) -> U + Send + 'static {

        match self.sender.send(Box::new(v)) {
            Ok(()) => {
                Ok(())
            }
            Err(error) => {
                Err(SendError(error))
            }
        }
    }
}

impl<T, U> Clone for Sender<T, U> {
    fn clone(&self) -> Self {
        Self {sender: self.sender.clone()}
    }
}

pub struct SendError<T, U>(pub MpscSendError<Box<dyn FnOnce(&mut T) -> U + Send>>);

impl<T, U> Debug for SendError<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}