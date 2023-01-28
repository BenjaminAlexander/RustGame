use std::ops::ControlFlow;
use crate::threading::channel::ReceiveMetaData;
use crate::threading::listener::{ListenedOrDidNotListen, ChannelEvent};

pub type ListenResult<T> = ControlFlow<<T as ListenerTrait>::ThreadReturn, ListenedOrDidNotListen<T>>;
pub type ListenerEventResult<T> = ControlFlow<<T as ListenerTrait>::ThreadReturn, T>;

pub trait ListenerTrait: Send + Sized + 'static {
    type Event: Send + 'static;
    type ThreadReturn: Send + 'static;
    type ListenFor: Send + 'static;

    fn listen(self) -> ListenResult<Self>;

    fn on_channel_event(self, event: ChannelEvent<Self>) -> ListenerEventResult<Self>;

    fn on_stop(self, receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn;
}