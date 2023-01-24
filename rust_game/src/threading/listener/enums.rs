use crate::threading::channel::ReceiveMetaData;
use crate::threading::listener::{ListenedValueHolder, ListenerTrait};

pub enum ListenedOrDidNotListen<T: ListenerTrait> {
    Listened(T, T::ListenFor),
    DidNotListen(T)
}

pub enum ChannelEvent<T: ListenerTrait> {
    ChannelEmptyAfterListen(ListenedValueHolder<T>),
    ReceivedEvent(ReceiveMetaData, T::Event),
    ChannelDisconnected
}