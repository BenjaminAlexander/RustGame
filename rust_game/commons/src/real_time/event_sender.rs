use crate::real_time::{EventOrStopThread, Sender};

pub struct EventSender<T: Send> {
    sender: Sender<EventOrStopThread<T>>,
}

impl<T: Send> EventSender<T> {
    //TODO: find all pub(crate) in real_time and make them private in real_time
    pub(crate) fn new(sender: Sender<EventOrStopThread<T>>) -> Self {
        return Self { sender };
    }

    pub fn send_event(&self, event: T) -> Result<(), T> {
        return match self.sender.send(EventOrStopThread::Event(event)) {
            Ok(_) => Ok(()),
            Err(EventOrStopThread::Event(event)) => Err(event),
            _ => panic!("Unreachable"),
        };
    }

    pub fn send_stop_thread(&self) -> Result<(), ()> {
        return match self.sender.send(EventOrStopThread::StopThread) {
            Ok(_) => Ok(()),
            Err(EventOrStopThread::StopThread) => Err(()),
            _ => panic!("Unreachable"),
        };
    }
}

impl<T: Send> Clone for EventSender<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}
