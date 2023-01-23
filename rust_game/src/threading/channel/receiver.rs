use std::sync::mpsc;
use crate::threading::channel::SentValueHolder;

pub type TryRecvError = mpsc::TryRecvError;

pub type RecvError = mpsc::RecvError;

pub struct ReceivedValueHolder<T> {
    pub(super) sent_value_holder: SentValueHolder<T>
}

impl<T> ReceivedValueHolder<T> {

    pub fn get_message(&self) -> &T { &self.sent_value_holder.value }

    pub fn move_message(self) -> T { self.sent_value_holder.value }
}

pub struct Receiver<T> {
    pub(super) receiver: mpsc::Receiver<SentValueHolder<T>>
}

impl<T> Receiver<T> {

    pub fn try_recv_holder(&self) -> Result<ReceivedValueHolder<T>, TryRecvError> {
        return Ok(ReceivedValueHolder {
            sent_value_holder: self.receiver.try_recv()?
        });
    }

    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        return Ok(self.try_recv_holder()?.move_message());
    }

    pub fn recv_holder(&self) -> Result<ReceivedValueHolder<T>, RecvError> {
        return Ok(ReceivedValueHolder {
            sent_value_holder: self.receiver.recv()?
        });
    }

    pub fn recv(&self) -> Result<T, RecvError> {
        return Ok(self.recv_holder()?.move_message());
    }
}
