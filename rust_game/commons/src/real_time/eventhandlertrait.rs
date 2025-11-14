use crate::{real_time::ReceiveMetaData, time::TimeDuration};

pub enum EventHandleResult<T: EventHandlerTrait> {
    WaitForNextEvent,
    WaitForNextEventOrTimeout(TimeDuration),
    TryForNextEvent,
    StopThread(T::ThreadReturn),
}

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

    //TODO: add more sane defaults for stop
    fn on_stop(self, receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn;
}
