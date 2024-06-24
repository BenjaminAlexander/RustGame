use crate::threading::eventhandling::EventOrStopThread;
use crate::threading::eventhandling::EventSenderTrait;

pub trait SenderTrait<T>: Clone + Send {
    fn send(&self, value: T) -> Result<(), T>;
}

impl<T, U: SenderTrait<EventOrStopThread<T>>> EventSenderTrait<T> for U {
    fn send_event(&self, event: T) -> Result<(), T> {
        return match self.send(EventOrStopThread::Event(event)) {
            Ok(_) => Ok(()),
            Err(EventOrStopThread::Event(event)) => Err(event),
            _ => panic!("Unreachable"),
        };
    }

    fn send_stop_thread(&self) -> Result<(), ()> {
        return match self.send(EventOrStopThread::StopThread) {
            Ok(_) => Ok(()),
            Err(EventOrStopThread::StopThread) => Err(()),
            _ => panic!("Unreachable"),
        };
    }
}
