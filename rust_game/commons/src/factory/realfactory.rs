use std::time::{SystemTime, UNIX_EPOCH};
use crate::factory::FactoryTrait;
use crate::time::TimeValue;

#[derive(Clone, Copy)]
pub struct RealFactory {

}

impl FactoryTrait for RealFactory {
    fn now(&self) -> TimeValue {
        return TimeValue::from_seconds_since_epoch(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64());
    }
}