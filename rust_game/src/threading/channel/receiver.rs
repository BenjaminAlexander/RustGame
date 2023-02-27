use std::sync::mpsc;
use commons::stats::RollingStatsLogger;
use commons::time::TimeDuration;
use crate::threading::channel::{ReceiveMetaData, SendMetaData};

pub type TryRecvError = mpsc::TryRecvError;

pub type RecvError = mpsc::RecvError;

pub struct Receiver<T> {
    receiver: mpsc::Receiver<(SendMetaData, T)>,
    duration_in_queue_logger: RollingStatsLogger<TimeDuration>
}

impl<T> Receiver<T> {

    pub fn new(receiver: mpsc::Receiver<(SendMetaData, T)>) -> Self {
        return Self{
            receiver,
            duration_in_queue_logger: RollingStatsLogger::new(100, 3.5, TimeDuration::from_seconds(30.0))
        }
    }

    pub fn try_recv_meta_data(&mut self) -> Result<(ReceiveMetaData, T), TryRecvError> {
        let (send_meta_data, value) = self.receiver.try_recv()?;
        return Ok((self.make_receive_meta_data(send_meta_data), value));
    }

    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        let (_, value) = self.try_recv_meta_data()?;
        return Ok(value);
    }

    pub fn recv_meta_data(&mut self) -> Result<(ReceiveMetaData, T), RecvError> {
        let (send_meta_data, value) = self.receiver.recv()?;
        return Ok((self.make_receive_meta_data(send_meta_data), value));
    }

    pub fn recv(&mut self) -> Result<T, RecvError> {
        let (_, value) = self.recv_meta_data()?;
        return Ok(value);
    }

    fn make_receive_meta_data(&mut self, send_meta_data: SendMetaData) -> ReceiveMetaData {
        let receive_meta_data = ReceiveMetaData::new(send_meta_data);
        self.duration_in_queue_logger.add_value(receive_meta_data.get_duration_in_queue());
        return receive_meta_data;
    }
}
