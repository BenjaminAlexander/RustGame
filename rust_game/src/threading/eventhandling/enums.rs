use crate::threading::channel::ReceiveMetaData;
use crate::threading::eventhandling::eventhandlertrait::EventHandlerTrait;

pub enum ChannelEvent<T> {
    ReceivedEvent(ReceiveMetaData, T),
    ChannelEmpty,
    ChannelDisconnected
}

pub enum EventOrStopThread<T> {
    Event(T),
    StopThread
}

pub enum WaitOrTryForNextEvent<T: EventHandlerTrait> {
    WaitForNextEvent(T),
    TryForNextEvent(T)
}