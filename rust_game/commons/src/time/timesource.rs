use crate::time::TimeValue;

pub trait TimeSource: Clone + Send {

    fn now(&self) -> TimeValue;

}