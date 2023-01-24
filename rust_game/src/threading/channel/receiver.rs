use std::sync::mpsc;
use crate::threading::channel::{ReceiveMetaData, SendMetaData};

pub type TryRecvError = mpsc::TryRecvError;

pub type RecvError = mpsc::RecvError;

pub struct Receiver<T> {
    pub(super) receiver: mpsc::Receiver<(SendMetaData, T)>
}

impl<T> Receiver<T> {

    pub fn try_recv_meta_data(&self) -> Result<(ReceiveMetaData, T), TryRecvError> {
        let (send_meta_data, value) = self.receiver.try_recv()?;
        return Ok((ReceiveMetaData::new(send_meta_data), value));
    }

    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        let (_, value) = self.try_recv_meta_data()?;
        return Ok(value);
    }

    pub fn recv_meta_data(&self) -> Result<(ReceiveMetaData, T), RecvError> {
        let (send_meta_data, value) = self.receiver.recv()?;

        return Ok((ReceiveMetaData::new(send_meta_data), value));
    }

    pub fn recv(&self) -> Result<T, RecvError> {
        let (_, value) = self.recv_meta_data()?;
        return Ok(value);
    }
}
