use crate::time::TimeValue;

pub struct ListenMetaData {
    time_received: TimeValue
}

impl ListenMetaData {

    pub fn new() -> Self {
        Self {
            time_received: TimeValue::now()
        }
    }

    pub fn get_time_received(&self) -> TimeValue { self.time_received }
}