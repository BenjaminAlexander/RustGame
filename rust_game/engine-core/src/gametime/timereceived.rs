use commons::time::TimeValue;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct TimeReceived<T> {
    t: T,
    time_received: TimeValue,
}

impl<T> TimeReceived<T> {
    // pub fn now(t: T) -> Self {
    //     Self{t, time_received: TimeValue::now() }
    // }

    pub fn new(time_received: TimeValue, t: T) -> Self {
        Self { t, time_received }
    }

    pub fn get_time_received(&self) -> TimeValue {
        self.time_received
    }

    pub fn get(&self) -> &T {
        &self.t
    }
}

impl<T> Clone for TimeReceived<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            t: self.t.clone(),
            time_received: self.time_received.clone(),
        }
    }
}
