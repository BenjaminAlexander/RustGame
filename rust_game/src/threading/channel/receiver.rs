use std::sync::mpsc;
use log::info;
use crate::threading::channel::{ReceiveMetaData, SendMetaData};
use crate::TimeDuration;

pub type TryRecvError = mpsc::TryRecvError;

pub type RecvError = mpsc::RecvError;

pub struct Receiver<T> {
    receiver: mpsc::Receiver<(SendMetaData, T)>,
    max_duration_in_channel: Option<TimeDuration>
}

impl<T> Receiver<T> {

    pub fn new(receiver: mpsc::Receiver<(SendMetaData, T)>) -> Self {
        return Self{
            receiver,
            max_duration_in_channel: None
        }
    }

    fn make_result<U>(&mut self, result: Result<(SendMetaData, T), U>) -> Result<(ReceiveMetaData, T), U> {
        let (send_meta_data, value) = result?;
        let receive_meta_data = ReceiveMetaData::new(send_meta_data);

        let duration = receive_meta_data.get_time_in_channel();

        if let Some(current_max_duration) = self.max_duration_in_channel.as_ref() {

            if duration.get_millis() > current_max_duration.get_millis() {
                info!("New record for wait time in channel: {:?}", duration);
                self.max_duration_in_channel = Some(duration);
            }

        } else {
            info!("New record for wait time in channel: {:?}", duration);
            self.max_duration_in_channel = Some(duration);
        }

        return Ok((receive_meta_data, value));
    }

    pub fn try_recv_meta_data(&mut self) -> Result<(ReceiveMetaData, T), TryRecvError> {
        return self.make_result(self.receiver.try_recv());
    }

    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        let (_, value) = self.try_recv_meta_data()?;
        return Ok(value);
    }

    pub fn recv_meta_data(&mut self) -> Result<(ReceiveMetaData, T), RecvError> {
        return self.make_result(self.receiver.recv());
    }

    pub fn recv(&mut self) -> Result<T, RecvError> {
        let (_, value) = self.recv_meta_data()?;
        return Ok(value);
    }
}
