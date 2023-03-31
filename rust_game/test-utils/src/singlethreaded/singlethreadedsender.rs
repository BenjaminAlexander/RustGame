use std::sync::{Arc, Mutex};
use commons::threading::channel::{RealSender, SendError, SenderTrait};
use crate::singlethreaded::SingleThreadedFactory;

pub struct SingleThreadedSender<T: Send> {
    internal: Arc<Mutex<Internal<T>>>
}

struct Internal<T: Send> {
    sender: RealSender<SingleThreadedFactory, T>,
    on_send: Option<Box<dyn Fn() + Send >>
}

impl<T: Send> Drop for Internal<T> {
    fn drop(&mut self) {
        if let Some(on_send) = &self.on_send {
            on_send();
        }
    }
}

impl<T: Send> SingleThreadedSender<T> {
    pub fn new(sender: RealSender<SingleThreadedFactory, T>) -> Self {

        let internal = Internal {
            sender,
            on_send: None
        };

        return Self {
            internal: Arc::new(Mutex::new(internal))
        };
    }

    pub fn set_on_send(&self, function: impl Fn() + Send + 'static) {
        self.internal.lock().unwrap().on_send = Some(Box::new(function));
    }
}

impl<T: Send> SenderTrait<T> for SingleThreadedSender<T> {
    fn send(&self, value: T) -> Result<(), SendError<T>> {
        let internal = self.internal.lock().unwrap();
        let result = internal.sender.send(value);
        if let Some(on_send) = &internal.on_send {
            on_send();
        }
        return result;
    }
}

impl<T: Send> Clone for SingleThreadedSender<T> {
    fn clone(&self) -> Self {
        return Self {
            internal: self.internal.clone()
        };
    }
}