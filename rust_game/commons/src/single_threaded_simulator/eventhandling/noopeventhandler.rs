use crate::threading::channel::ReceiveMetaData;
use crate::threading::eventhandling::{
    ChannelEvent,
    EventHandleResult,
    EventHandlerTrait,
};

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

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> EventHandleResult<Self> {
        return match channel_event {
            ChannelEvent::ReceivedEvent(_, ()) => EventHandleResult::TryForNextEvent(self),
            ChannelEvent::Timeout => EventHandleResult::TryForNextEvent(self),
            ChannelEvent::ChannelEmpty => EventHandleResult::WaitForNextEvent(self),
            ChannelEvent::ChannelDisconnected => EventHandleResult::StopThread,
        };
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) {
        (self.on_stop_func)();
    }
}
