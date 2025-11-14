use std::sync::{
    Arc,
    Mutex,
};

use crate::real_time::simulation::sender_link::SenderLink;

pub struct SingleThreadedSender<T: Send> {
    link: Arc<Mutex<SenderLink<T>>>,
}

impl<T: Send> SingleThreadedSender<T> {
    pub fn new(link: SenderLink<T>) -> Self {
        return Self {
            link: Arc::new(Mutex::new(link)),
        };
    }

    pub fn send(&self, value: T) -> Result<(), T> {
        return self.link.lock().unwrap().send(value);
    }
}

impl<T: Send> Clone for SingleThreadedSender<T> {
    fn clone(&self) -> Self {
        return Self {
            link: self.link.clone(),
        };
    }
}