use crate::threading::channel::ReceiveMetaData;
use crate::threading::listener::{ListenerTrait, ListenMetaData};

pub enum ListenedOrDidNotListen<T: ListenerTrait> {
    Listened(T, T::ListenFor),
    DidNotListen(T)
}

pub enum ChannelEvent<T: ListenerTrait> {
    ChannelEmptyAfterListen(ListenMetaData, T::ListenFor),
    ReceivedEvent(ReceiveMetaData, T::Event),
    ChannelDisconnected
}