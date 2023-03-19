use crate::time::TimeValue;

pub trait TimeSource {

    fn now() -> TimeValue;

}