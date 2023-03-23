use crate::factory::FactoryTrait;
use crate::time::TimeValue;

pub struct ListenMetaData {
    time_received: TimeValue
}

impl ListenMetaData {

    pub fn new(factory: &impl FactoryTrait) -> Self {
        Self {
            time_received: factory.now()
        }
    }

    pub fn get_time_received(&self) -> TimeValue { self.time_received }
}