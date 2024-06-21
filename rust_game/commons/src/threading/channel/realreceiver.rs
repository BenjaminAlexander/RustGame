use crate::factory::FactoryTrait;
use crate::threading::channel::{ReceiveMetaData, ReceiverTrait, SendMetaData, TryRecvError};
use crate::time::TimeDuration;
use std::sync::mpsc;

pub type RecvError = mpsc::RecvError;

pub type RecvTimeoutError = mpsc::RecvTimeoutError;

pub struct RealReceiver<Factory: FactoryTrait, T: Send> {
    factory: Factory,
    receiver: mpsc::Receiver<(SendMetaData, T)>, //duration_in_queue_logger: RollingStatsLogger<TimeDuration>
}

impl<Factory: FactoryTrait, T: Send> ReceiverTrait<T> for RealReceiver<Factory, T> {
    fn try_recv_meta_data(&mut self) -> Result<(ReceiveMetaData, T), TryRecvError> {
        let (send_meta_data, value) = self.receiver.try_recv()?;
        return Ok((self.make_receive_meta_data(send_meta_data), value));
    }
}

impl<Factory: FactoryTrait, T: Send> RealReceiver<Factory, T> {
    pub fn new(factory: Factory, receiver: mpsc::Receiver<(SendMetaData, T)>) -> Self {
        return Self {
            factory,
            receiver, //duration_in_queue_logger: RollingStatsLogger::new(100, 3.5, TimeDuration::from_seconds(30.0))
        };
    }

    pub fn recv_meta_data(&mut self) -> Result<(ReceiveMetaData, T), RecvError> {
        let (send_meta_data, value) = self.receiver.recv()?;
        return Ok((self.make_receive_meta_data(send_meta_data), value));
    }

    pub fn recv(&mut self) -> Result<T, RecvError> {
        let (_, value) = self.recv_meta_data()?;
        return Ok(value);
    }

    pub fn recv_timeout_meta_data(
        &mut self,
        duration: TimeDuration,
    ) -> Result<(ReceiveMetaData, T), RecvTimeoutError> {
        if let Some(std_duration) = duration.to_duration() {
            let (send_meta_data, value) = self.receiver.recv_timeout(std_duration)?;
            return Ok((self.make_receive_meta_data(send_meta_data), value));
        } else {
            return Err(RecvTimeoutError::Timeout);
        }
    }

    pub fn recv_timeout(&mut self, duration: TimeDuration) -> Result<T, RecvTimeoutError> {
        let (_, value) = self.recv_timeout_meta_data(duration)?;
        return Ok(value);
    }

    fn make_receive_meta_data(&mut self, send_meta_data: SendMetaData) -> ReceiveMetaData {
        let receive_meta_data = ReceiveMetaData::new(&self.factory, send_meta_data);
        //self.duration_in_queue_logger.add_value(receive_meta_data.get_duration_in_queue());
        return receive_meta_data;
    }
}
