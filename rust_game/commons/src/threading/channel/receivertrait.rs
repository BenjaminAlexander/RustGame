use std::sync::mpsc::TryRecvError;

use crate::threading::channel::ReceiveMetaData;

pub trait ReceiverTrait<T: Send> {
    fn try_recv_meta_data(&mut self) -> Result<(ReceiveMetaData, T), TryRecvError>;

    fn try_recv(&mut self) -> Result<T, TryRecvError> {
        let (_, value) = self.try_recv_meta_data()?;
        return Ok(value);
    }
}
