use crate::time::TimeValue;

pub struct SendMetaData {
    time_sent: TimeValue
}

impl SendMetaData {

    pub fn new() -> Self {
        return SendMetaData {
            time_sent: TimeValue::now()
        };
    }

    pub fn get_time_sent(&self) -> &TimeValue {
        return &self.time_sent;
    }
}