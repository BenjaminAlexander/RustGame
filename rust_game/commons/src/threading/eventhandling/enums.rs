use crate::threading::channel::ReceiveMetaData;
use crate::threading::eventhandling::eventhandlertrait::EventHandlerTrait;
use crate::time::TimeDuration;

pub enum ChannelEvent<T> {
    ReceivedEvent(ReceiveMetaData, T),
    Timeout,
    ChannelEmpty,
    ChannelDisconnected,
}

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
