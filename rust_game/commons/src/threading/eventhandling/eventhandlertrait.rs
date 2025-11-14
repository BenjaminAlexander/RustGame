use crate::{real_time::ReceiveMetaData, threading::eventhandling::EventHandleResult};

//TODO: rename this trait
pub trait EventHandlerTrait: Send + Sized + 'static {
    type Event: Send + 'static;
    type ThreadReturn: Send + 'static;

    fn on_event(&mut self, receive_meta_data: ReceiveMetaData, event: Self::Event) -> EventHandleResult<Self>;

    fn on_timeout(&mut self) -> EventHandleResult<Self> {
        return EventHandleResult::WaitForNextEvent;
    }

    fn on_channel_empty(&mut self) -> EventHandleResult<Self> {
        return EventHandleResult::WaitForNextEvent;
    }

    fn on_channel_disconnect(&mut self) -> EventHandleResult<Self>;

    fn on_stop(self, receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn;
}
