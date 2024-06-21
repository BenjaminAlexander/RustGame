use crate::threading::eventhandling::EventHandlerSendResult;

pub trait EventSenderTrait<T> {
    fn send_event(&self, event: T) -> EventHandlerSendResult<T>;

    fn send_stop_thread(&self) -> EventHandlerSendResult<T>;
}
