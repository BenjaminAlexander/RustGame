use std::ops::ControlFlow;
use std::ops::ControlFlow::Continue;
use crate::threading::eventhandling;
use crate::threading::eventhandling::{ChannelEventResult, EventHandlerTrait};
use crate::threading::eventhandling::WaitOrTryForNextEvent::TryForNextEvent;
use crate::threading::listener::{ListenedValueHolder, ChannelEvent, ListenerTrait};
use crate::threading::listener::ChannelEvent::{ChannelDisconnected, ChannelEmptyAfterListen, ReceivedEvent};
use crate::threading::listener::eventhandler::ListenerState::{ReadyToListen, WaitingForChannelEmptyAfterListen};
use crate::threading::listener::ListenedOrDidNotListen::{DidNotListen, Listened};

type EventResult<T> = ControlFlow<T::ThreadReturn, ListenerState<T>>;

//TODO: make struct
pub(super) enum ListenerState<T: ListenerTrait> {
    WaitingForChannelEmptyAfterListen(T, ListenedValueHolder<T>),
    ReadyToListen(T),
}

impl<T: ListenerTrait> EventHandlerTrait for ListenerState<T> {
    type Event = T::Event;
    type ThreadReturn = T::ThreadReturn;

    fn on_channel_event(mut self, event: eventhandling::ChannelEvent<Self>) -> ChannelEventResult<Self> {

        match event {
            eventhandling::ChannelEvent::ReceivedEvent(message) => {
                return Continue(TryForNextEvent(self.on_channel_event(ReceivedEvent(message))?));
            }
            eventhandling::ChannelEvent::ChannelEmpty => {
                return Continue(TryForNextEvent(self.listen()?));
            }
            eventhandling::ChannelEvent::ChannelDisconnected => {

                self = self.on_channel_event(ChannelDisconnected)?;

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
                    listener.on_channel_event(ChannelEmptyAfterListen(heard_value))?,
                ReadyToListen(listener) => listener
            }.listen()?
        {
            Listened(listener, value) => WaitingForChannelEmptyAfterListen(listener, ListenedValueHolder { value }),
            DidNotListen(listener) => ReadyToListen(listener)
        });
    }

    fn on_channel_event(self, event: ChannelEvent<T>) -> EventResult<T> {
        return Continue(match self {
            WaitingForChannelEmptyAfterListen(listener, listened_value_holder) =>
                WaitingForChannelEmptyAfterListen(listener.on_channel_event(event)?, listened_value_holder),
            ReadyToListen(listener) => ReadyToListen(listener.on_channel_event(event)?)
        });
    }
}