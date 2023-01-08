use crate::threading::eventhandling::eventhandlertrait::EventHandlerTrait;
use crate::threading::eventhandling::receivedeventholder::ReceivedEventHolder;
use crate::threading::eventhandling::SentEventHolder;

pub enum ChannelEvent<T: EventHandlerTrait> {
    ReceivedEvent(ReceivedEventHolder<T>),
    ChannelEmpty,
    ChannelDisconnected
}

pub enum EventOrStopThread<T: EventHandlerTrait> {
    Event(SentEventHolder<T>),
    StopThread
}

pub enum WaitOrTryForNextEvent<T: EventHandlerTrait> {
    WaitForNextEvent(T),
    TryForNextEvent(T)
}