use std::ops::ControlFlow;
use std::ops::ControlFlow::Continue;
use crate::threading::eventhandling::{ChannelEvent as EventChannelEvent, ChannelEventResult, EventHandlerTrait, WaitOrTryForNextEvent};
use crate::threading::listener::{ListenedOrDidNotListen, ListenedValueHolder, ChannelEvent, ListenerTrait};
use crate::threading::listener::ListenedOrDidNotListen::{DidNotListen, Listened};
use crate::threading::listener::ListenerState::{ReadyToListen, WaitingForChannelEmptyAfterListen};

type EventResult<T: ListenerTrait> = ControlFlow<T::ThreadReturn, ListenerState<T>>;

//TODO: reduce visability to pub(super)
//TODO: make struct
pub enum ListenerState<T: ListenerTrait> {
    WaitingForChannelEmptyAfterListen(T, ListenedValueHolder<T>),
    ReadyToListen(T),
}

impl<T: ListenerTrait> EventHandlerTrait for ListenerState<T> {
    type Event = T::Event;
    type ThreadReturn = T::ThreadReturn;

    fn on_channel_event(mut self, event: EventChannelEvent<Self>) -> ChannelEventResult<Self> {

        match event {
            EventChannelEvent::ReceivedEvent(message) => {
                return Continue(WaitOrTryForNextEvent::TryForNextEvent(self.on_event(ChannelEvent::ReceivedEvent(message))?));
            }
            EventChannelEvent::ChannelEmpty => {
                return Continue(WaitOrTryForNextEvent::TryForNextEvent(self.listen()?));
            }
            EventChannelEvent::ChannelDisconnected => {

                self = self.on_event(ChannelEvent::ChannelDisconnected)?;

                loop {
                    self = self.listen()?;
                }
            }
        };
    }

    fn on_stop(self) -> Self::ThreadReturn {
        match self {
            WaitingForChannelEmptyAfterListen(listener, _) => listener.on_stop(),
            ReadyToListen(listener) => listener.on_stop()
        }
    }
}

impl<T: ListenerTrait> ListenerState<T> {

    fn listen(self) -> EventResult<T> {
        return Continue(match
            match self {
                WaitingForChannelEmptyAfterListen(listener, heard_value) =>
                    listener.on_channel_event(ChannelEvent::ChannelEmptyAfterListen(heard_value))?,
                ReadyToListen(listener) => listener
            }.listen()?
        {
            Listened(listener, value) => WaitingForChannelEmptyAfterListen(listener, ListenedValueHolder { value }),
            DidNotListen(listener) => ReadyToListen(listener)
        });
    }

    fn on_event(self, event: ChannelEvent<T>) -> EventResult<T> {
        return Continue(match self {
            WaitingForChannelEmptyAfterListen(listener, listened_value_holder) =>
                WaitingForChannelEmptyAfterListen(listener.on_channel_event(event)?, listened_value_holder),
            ListenerState::ReadyToListen(listener) => ReadyToListen(listener.on_channel_event(event)?)
        });
    }
}