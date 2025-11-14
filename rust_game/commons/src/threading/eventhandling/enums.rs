use crate::real_time::ReceiveMetaData;
use crate::threading::eventhandling::eventhandlertrait::EventHandlerTrait;
use crate::time::TimeDuration;

//TODO: hide this enum
pub enum ChannelEvent<T> {
    ReceivedEvent(ReceiveMetaData, T),
    Timeout,
    ChannelEmpty,
    ChannelDisconnected,
}

impl<T> ChannelEvent<T> {

    pub fn handle<U: EventHandlerTrait<Event = T>>(
        self,
        message_handler: &mut U,
    ) -> EventHandleResult<U> {
        match self {
            ChannelEvent::ReceivedEvent(receive_meta_data, event) => message_handler.on_event(receive_meta_data, event),
            ChannelEvent::Timeout => message_handler.on_timeout(),
            ChannelEvent::ChannelEmpty => message_handler.on_channel_empty(),
            ChannelEvent::ChannelDisconnected => message_handler.on_channel_disconnect(),
        }
    }
}

//TODO: hide EventOrStopThread enum
pub enum EventOrStopThread<T> {
    Event(T),
    StopThread,
}

pub enum EventHandleResult<T: EventHandlerTrait> {
    WaitForNextEvent,
    WaitForNextEventOrTimeout(TimeDuration),
    TryForNextEvent,
    StopThread(T::ThreadReturn),
}
