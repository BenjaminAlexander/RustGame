use crate::threading::channel::{
    SendMetaData,
    SenderTrait,
};
use crate::time::TimeSource;
use std::sync::mpsc::{
    self,
    SendError,
};

pub struct RealSender<T: Send> {
    time_source: TimeSource,
    sender: mpsc::Sender<(SendMetaData, T)>,
}

impl<T: Send> RealSender<T> {
    pub fn new(time_source: TimeSource, sender: mpsc::Sender<(SendMetaData, T)>) -> Self {
        return Self { time_source, sender };
    }
}

impl<T: Send> SenderTrait<T> for RealSender<T> {
    fn send(&self, value: T) -> Result<(), T> {
        let send_meta_data = SendMetaData::new(&self.time_source);

        return match self.sender.send((send_meta_data, value)) {
            Ok(()) => Result::Ok(()),
            Err(SendError((_, value))) => Result::Err(value),
        };
    }
}

impl<T: Send> Clone for RealSender<T> {
    fn clone(&self) -> Self {
        return Self {
            time_source: self.time_source.clone(),
            sender: self.sender.clone(),
        };
    }
}
