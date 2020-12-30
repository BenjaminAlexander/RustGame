use std::time::SystemTime;

#[derive(Debug)]
pub struct TimeReceived<T> {
    t: T,
    time_received: SystemTime
}

impl<T> TimeReceived<T> {
    pub fn now(t: T) -> Self {
        Self{t, time_received: SystemTime::now() }
    }

    pub fn new(time_received: SystemTime, t: T) -> Self {
        Self{ t, time_received }
    }
}

impl<T> Clone for TimeReceived<T>
    where T: Clone {

    fn clone(&self) -> Self {
        Self {
            t: self.t.clone(),
            time_received: self.time_received.clone()
        }
    }
}