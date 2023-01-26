use std::sync::mpsc;
use crate::threading::channel::SendMetaData;

pub type SendError<T> = mpsc::SendError<(SendMetaData, T)>;

pub struct Sender<T> {
    sender: mpsc::Sender<(SendMetaData, T)>
}

impl<T> Sender<T> {

    pub fn new(sender: mpsc::Sender<(SendMetaData, T)>) -> Self {
        return Self{
            sender
        }
    }

    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        return self.sender.send((SendMetaData::new(), value));
    }

}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self { sender: self.sender.clone() }
    }
}
