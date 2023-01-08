use crate::threading::eventhandling::ReceivedEventHolder;
use crate::threading::listener::{ListenedValueHolder, ListenerState, ListenerTrait};

pub enum ListenedOrDidNotListen<T: ListenerTrait> {
    Listened(T, T::ListenFor),
    DidNotListen(T)
}

pub enum ChannelEvent<T: ListenerTrait> {
    ChannelEmptyAfterListen(ListenedValueHolder<T>),
    ReceivedEvent(ReceivedEventHolder<ListenerState<T>>),
    ChannelDisconnected
}