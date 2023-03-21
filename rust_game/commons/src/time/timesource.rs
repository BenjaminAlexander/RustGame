use crate::time::TimeValue;

pub trait TimeSource: Default + Clone + Send {

    fn now(&self) -> TimeValue;

}