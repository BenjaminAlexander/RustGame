use crate::singlethreaded::channel::senderlink::SenderLink;
use commons::threading::channel::SenderTrait;
use std::sync::{
    Arc,
    Mutex,
};

pub struct SingleThreadedSender<T: Send> {
    link: Arc<Mutex<SenderLink<T>>>,
}

impl<T: Send> SingleThreadedSender<T> {
    pub fn new(link: SenderLink<T>) -> Self {
        return Self {
            link: Arc::new(Mutex::new(link)),
        };
    }
}

impl<T: Send> Clone for SingleThreadedSender<T> {
    fn clone(&self) -> Self {
        return Self {
            link: self.link.clone(),
        };
    }
}

impl<T: Send> SenderTrait<T> for SingleThreadedSender<T> {
    fn send(&self, value: T) -> Result<(), T> {
        return self.link.lock().unwrap().send(value);
    }
}
