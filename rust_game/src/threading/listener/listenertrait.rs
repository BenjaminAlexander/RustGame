use std::ops::ControlFlow;
use crate::threading::listener::{ListenedOrDidNotListen, ChannelEvent, ListenerState};

pub type ListenResult<T: ListenerTrait> = ControlFlow<T::ThreadReturn, ListenedOrDidNotListen<T>>;
pub type ListenerEventResult<T: ListenerTrait> = ControlFlow<T::ThreadReturn, T>;

pub trait ListenerTrait: Send + Sized + 'static {
    type Event: Send + 'static;
    type ThreadReturn: Send + 'static;
    type ListenFor: Send + 'static;

    //TODO: make this a static method, replace with build thread
    fn to_message_handler(self) -> ListenerState<Self> {
        return ListenerState::ReadyToListen(self);
    }

    fn listen(self) -> ListenResult<Self>;

    fn on_channel_event(self, event: ChannelEvent<Self>) -> ListenerEventResult<Self>;

    fn on_stop(self) -> Self::ThreadReturn;
}