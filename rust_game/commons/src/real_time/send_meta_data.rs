use crate::{real_time::TimeSource, time::TimeValue};

#[derive(Debug, Clone, Copy)]
pub struct SendMetaData {
    time_sent: TimeValue,
}

impl SendMetaData {
    pub fn new(time_source: &TimeSource) -> Self {
        return SendMetaData {
            time_sent: time_source.now(),
        };
    }

    pub fn get_time_sent(&self) -> &TimeValue {
        return &self.time_sent;
    }
}
