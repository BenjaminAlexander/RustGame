use std::sync::mpsc;
use crate::factory::FactoryTrait;
use crate::threading::channel::{ReceiveMetaData, SendMetaData};
use crate::time::TimeDuration;

pub type TryRecvError = mpsc::TryRecvError;

pub type RecvError = mpsc::RecvError;

pub type RecvTimeoutError = mpsc::RecvTimeoutError;

pub struct Receiver<T> {
    receiver: mpsc::Receiver<(SendMetaData, T)>
    //duration_in_queue_logger: RollingStatsLogger<TimeDuration>
}

impl<T> Receiver<T> {

    pub fn new(receiver: mpsc::Receiver<(SendMetaData, T)>) -> Self {
        return Self{
            receiver
            //duration_in_queue_logger: RollingStatsLogger::new(100, 3.5, TimeDuration::from_seconds(30.0))
        }
    }

    pub fn try_recv_meta_data(&mut self, factory: &impl FactoryTrait) -> Result<(ReceiveMetaData, T), TryRecvError> {
        let (send_meta_data, value) = self.receiver.try_recv()?;
        return Ok((self.make_receive_meta_data(factory, send_meta_data), value));
    }

    pub fn try_recv(&mut self, factory: &impl FactoryTrait) -> Result<T, TryRecvError> {
        let (_, value) = self.try_recv_meta_data(factory)?;
        return Ok(value);
    }

    pub fn recv_meta_data(&mut self, factory: &impl FactoryTrait) -> Result<(ReceiveMetaData, T), RecvError> {
        let (send_meta_data, value) = self.receiver.recv()?;
        return Ok((self.make_receive_meta_data(factory, send_meta_data), value));
    }

    pub fn recv(&mut self, factory: &impl FactoryTrait) -> Result<T, RecvError> {
        let (_, value) = self.recv_meta_data(factory)?;
        return Ok(value);
    }

    pub fn recv_timeout_meta_data(&mut self, factory: &impl FactoryTrait, duration: TimeDuration) -> Result<(ReceiveMetaData, T), RecvTimeoutError> {
        let (send_meta_data, value) = self.receiver.recv_timeout(duration.to_std())?;
        return Ok((self.make_receive_meta_data(factory, send_meta_data), value));
    }

    pub fn recv_timeout(&mut self, factory: &impl FactoryTrait, duration: TimeDuration) -> Result<T, RecvTimeoutError> {
        let (_, value) = self.recv_timeout_meta_data(factory, duration)?;
        return Ok(value);
    }

    fn make_receive_meta_data(&mut self, factory: &impl FactoryTrait, send_meta_data: SendMetaData) -> ReceiveMetaData {
        let receive_meta_data = ReceiveMetaData::new(factory, send_meta_data);
        //self.duration_in_queue_logger.add_value(receive_meta_data.get_duration_in_queue());
        return receive_meta_data;
    }
}
