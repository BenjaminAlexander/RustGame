use crate::factory::FactoryTrait;
use crate::time::{TimeDuration, TimeValue};
use crate::threading::channel::SendMetaData;

pub struct ReceiveMetaData {
    send_meta_data: SendMetaData,
    time_received: TimeValue
}

impl ReceiveMetaData {

    pub fn new(factory: &impl FactoryTrait, send_meta_data: SendMetaData) -> Self {
        return ReceiveMetaData {
            send_meta_data,
            time_received: factory.now()
        }
    }

    pub fn get_send_meta_data(&self) -> &SendMetaData {
        return &self.send_meta_data;
    }

    pub fn get_time_received(&self) -> &TimeValue {
        return &self.time_received;
    }

    pub fn get_duration_in_queue(&self) -> TimeDuration {
        return self.time_received.duration_since(self.send_meta_data.get_time_sent());
    }
}
