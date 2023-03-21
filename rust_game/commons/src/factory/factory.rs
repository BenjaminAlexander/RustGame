use crate::time::TimeSource;

pub trait Factory {
    type TimeSource: TimeSource;

    fn new_time_source() -> Self::TimeSource {
        return Self::TimeSource::default();
    }
}