use crate::real_time::{EventHandleResult, EventHandlerTrait, ReceiveMetaData};

pub struct NoOpEventHandler {
    on_stop_func: Box<dyn FnOnce() + Send + 'static>,
}

impl NoOpEventHandler {
    pub fn new(on_stop: impl FnOnce() + Send + 'static) -> Self {
        return Self {
            on_stop_func: Box::new(on_stop),
        };
    }
}

impl EventHandlerTrait for NoOpEventHandler {
    type Event = ();
    type ThreadReturn = ();

    fn on_stop(self, _: ReceiveMetaData) -> Self::ThreadReturn {
        (self.on_stop_func)();
    }
    
    fn on_event(&mut self, _: ReceiveMetaData, _: Self::Event) -> EventHandleResult<Self> {
        EventHandleResult::TryForNextEvent
    }

    fn on_channel_disconnect(&mut self) -> EventHandleResult<Self> {
        EventHandleResult::StopThread(())
    }
}
