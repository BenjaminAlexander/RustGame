use crate::threading::channel::ReceiveMetaData;
use crate::threading::eventhandling::{
    ChannelEvent,
    EventHandleResult,
};

pub trait EventHandlerTrait: Send + Sized + 'static {
    type Event: Send + 'static;

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> EventHandleResult<Self>;

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) {
        //no-op default implementation
    }
}
