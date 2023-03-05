use std::ops::ControlFlow;
use std::ops::ControlFlow::Continue;
use crate::threading::channel::ReceiveMetaData;
use crate::threading::eventhandling;
use crate::threading::eventhandling::{ChannelEventResult, EventHandlerTrait};
use crate::threading::eventhandling::WaitOrTryForNextEvent::TryForNextEvent;
use crate::threading::listener::{ChannelEvent, ListenerTrait, ListenMetaData};
use crate::threading::listener::ChannelEvent::{ChannelDisconnected, ChannelEmptyAfterListen, ReceivedEvent};
use crate::threading::listener::eventhandler::ListenerState::{ReadyToListen, WaitingForChannelEmptyAfterListen};
use crate::threading::listener::ListenedOrDidNotListen::{DidNotListen, Listened};

type EventResult<T> = ControlFlow<<T as ListenerTrait>::ThreadReturn, ListenerState<T>>;

pub enum ListenerState<T: ListenerTrait> {
    WaitingForChannelEmptyAfterListen(T, ListenMetaData, T::ListenFor),
    ReadyToListen(T),
}

impl<T: ListenerTrait> EventHandlerTrait for ListenerState<T> {
    type Event = T::Event;
    type ThreadReturn = T::ThreadReturn;

    fn on_channel_event(mut self, event: eventhandling::ChannelEvent<Self::Event>) -> ChannelEventResult<Self> {

        match event {
            eventhandling::ChannelEvent::ReceivedEvent(receive_meta_data, event) => {
                return Continue(TryForNextEvent(self.on_channel_event(ReceivedEvent(receive_meta_data, event))?));
            }
            eventhandling::ChannelEvent::ChannelEmpty => {
                return Continue(TryForNextEvent(self.listen()?));
            }
            eventhandling::ChannelEvent::Timeout => {
                return Continue(TryForNextEvent(self));
            }
            eventhandling::ChannelEvent::ChannelDisconnected => {

                self = self.on_channel_event(ChannelDisconnected)?;

                loop {
                    self = self.listen()?;
                }
            }
        };
    }

    fn on_stop(self, receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        match self {
            WaitingForChannelEmptyAfterListen(listener, _, _) => listener.on_stop(receive_meta_data),
            ReadyToListen(listener) => listener.on_stop(receive_meta_data)
        }
    }
}

impl<T: ListenerTrait> ListenerState<T> {

    fn listen(self) -> EventResult<T> {
        return Continue(match
            match self {
                WaitingForChannelEmptyAfterListen(listener, listen_meta_data, value) =>
                    listener.on_channel_event(ChannelEmptyAfterListen(listen_meta_data, value))?,
                ReadyToListen(listener) => listener
            }.listen()?
        {
            Listened(listener, value) => WaitingForChannelEmptyAfterListen(listener, ListenMetaData::new(), value),
            DidNotListen(listener) => ReadyToListen(listener)
        });
    }

    fn on_channel_event(self, event: ChannelEvent<T>) -> EventResult<T> {
        return Continue(match self {
            WaitingForChannelEmptyAfterListen(listener, listen_meta_data, value) =>
                WaitingForChannelEmptyAfterListen(listener.on_channel_event(event)?, listen_meta_data, value),
            ReadyToListen(listener) => ReadyToListen(listener.on_channel_event(event)?)
        });
    }
}