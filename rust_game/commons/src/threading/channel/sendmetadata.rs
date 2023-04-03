use crate::factory::FactoryTrait;
use crate::time::TimeValue;

#[derive(Debug)]
pub struct SendMetaData {
    time_sent: TimeValue
}

impl SendMetaData {

    pub fn new(factory: &impl FactoryTrait) -> Self {
        return SendMetaData {
            time_sent: factory.now()
        };
    }

    pub fn get_time_sent(&self) -> &TimeValue {
        return &self.time_sent;
    }
}