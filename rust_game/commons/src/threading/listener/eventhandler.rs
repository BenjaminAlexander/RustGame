use std::ops::ControlFlow;
use std::ops::ControlFlow::Continue;
use crate::factory::FactoryTrait;
use crate::threading::channel::ReceiveMetaData;
use crate::threading::eventhandling;
use crate::threading::eventhandling::{ChannelEventResult, EventHandlerTrait};
use crate::threading::eventhandling::WaitOrTryForNextEvent::TryForNextEvent;
use crate::threading::listener::{ChannelEvent, ListenerTrait, ListenMetaData};
use crate::threading::listener::ChannelEvent::{ChannelDisconnected, ChannelEmptyAfterListen, ReceivedEvent};
use crate::threading::listener::eventhandler::InternalState::{ReadyToListen, WaitingForChannelEmptyAfterListen};
use crate::threading::listener::ListenedOrDidNotListen::{DidNotListen, Listened};

type EventResult<Factory, T> = ControlFlow<<T as ListenerTrait>::ThreadReturn, ListenerState<Factory, T>>;

pub struct ListenerState<Factory: FactoryTrait, T: ListenerTrait> {
    listener: T,
    factory: Factory,
    state: InternalState<T>
}

enum InternalState<T: ListenerTrait> {
    WaitingForChannelEmptyAfterListen(ListenMetaData, T::ListenFor),
    ReadyToListen,
}

impl<Factory: FactoryTrait, T: ListenerTrait> EventHandlerTrait for ListenerState<Factory, T> {
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
        return self.listener.on_stop(receive_meta_data);
    }
}

impl<Factory: FactoryTrait, T: ListenerTrait> ListenerState<Factory, T> {

    fn listen(mut self) -> EventResult<Factory, T> {
        return Continue(match
            match self.state {
                WaitingForChannelEmptyAfterListen(listen_meta_data, value) =>
                    self.listener.on_channel_event(ChannelEmptyAfterListen(listen_meta_data, value))?,
                ReadyToListen => self.listener
            }.listen()?
        {
            Listened(listener, value) => {
                self.listener = listener;
                self.state = WaitingForChannelEmptyAfterListen(ListenMetaData::new(&self.factory), value);
                self
            },
            DidNotListen(listener) => {
                self.listener = listener;
                self.state = ReadyToListen;
                self
            }
        });
    }

    fn on_channel_event(mut self, event: ChannelEvent<T>) -> EventResult<Factory, T> {
        self.listener = self.listener.on_channel_event(event)?;
        return Continue(self);
    }
}