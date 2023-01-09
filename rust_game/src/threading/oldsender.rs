use std::sync::mpsc::{Sender as MpscSender, SendError as MpscSendError};
use core::fmt::Debug;
use serde::export::Formatter;
use core::fmt;

pub struct OldSender<T, U> {
    sender: MpscSender<Box<dyn FnOnce(&mut T) -> U + Send + 'static>>
}

impl<T, U> OldSender<T, U> {

    pub fn new(sender: MpscSender<Box<dyn FnOnce(&mut T) -> U + Send + 'static>>) -> Self {
        OldSender {sender}
    }

    //TODO: Make this a Custom ResultType
    pub fn send<V>(&self, v: V) -> Result<(), OldSendError<T, U>>
        where V: FnOnce(&mut T) -> U + Send + 'static {

        match self.sender.send(Box::new(v)) {
            Ok(()) => {
                Ok(())
            }
            Err(error) => {
                Err(OldSendError(error))
            }
        }
    }
}

impl<T, U> Clone for OldSender<T, U> {
    fn clone(&self) -> Self {
        Self {sender: self.sender.clone()}
    }
}

pub struct OldSendError<T, U>(pub MpscSendError<Box<dyn FnOnce(&mut T) -> U + Send>>);

impl<T, U> Debug for OldSendError<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}