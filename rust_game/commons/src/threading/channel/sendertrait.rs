use crate::threading::eventhandling::EventOrStopThread;
use crate::threading::eventhandling::EventSenderTrait;

pub trait SenderTrait<T>: Clone + Send {
    fn send(&self, value: T) -> Result<(), T>;
}
