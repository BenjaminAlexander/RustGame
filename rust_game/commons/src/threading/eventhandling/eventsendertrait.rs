use crate::threading::eventhandling::SendResult;

pub trait EventSenderTrait<T> {

    fn send_event(&self, event: T) -> SendResult<T>;

    fn send_stop_thread(&self) -> SendResult<T>;
}