use crate::{
    real_time::ReceiveMetaData,
    time::TimeDuration,
};

pub enum EventHandleResult {
    WaitForNextEvent,
    WaitForNextEventOrTimeout(TimeDuration),
    TryForNextEvent,
    StopThread,
}

pub trait HandleEvent: Send + Sized + 'static {
    type Event: Send + 'static;
    type ThreadReturn: Send + 'static;

    fn on_event(
        &mut self,
        receive_meta_data: ReceiveMetaData,
        event: Self::Event,
    ) -> EventHandleResult;

    fn on_timeout(&mut self) -> EventHandleResult {
        return EventHandleResult::WaitForNextEvent;
    }

    fn on_channel_empty(&mut self) -> EventHandleResult {
        return EventHandleResult::WaitForNextEvent;
    }

    fn on_channel_disconnect(&mut self) -> EventHandleResult {
        return EventHandleResult::StopThread;
    }

    fn on_stop_remote(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        return self.on_stop_self();
    }

    fn on_stop_self(self) -> Self::ThreadReturn;
}
