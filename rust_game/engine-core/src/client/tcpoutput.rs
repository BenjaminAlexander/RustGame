use std::ops::ControlFlow::{Break, Continue};
use commons::threading::channel::ReceiveMetaData;
use commons::threading::eventhandling::{ChannelEvent, EventHandleResult, EventHandlerTrait};
use commons::threading::eventhandling::ChannelEvent::{ReceivedEvent, ChannelEmpty, ChannelDisconnected, Timeout};
use commons::threading::eventhandling::WaitOrTryForNextEvent::{TryForNextEvent, WaitForNextEvent};
use crate::interface::{GameFactoryTrait, TcpWriter};

//TODO: Send response to time messages to calculate ping
pub struct TcpOutput<GameFactory: GameFactoryTrait> {
    tcp_sender: TcpWriter<GameFactory>
}

impl<GameFactory: GameFactoryTrait> TcpOutput<GameFactory> {

    pub fn new(tcp_sender: TcpWriter<GameFactory>) -> Self {
        return Self{
            tcp_sender
        };
    }
}

impl<GameFactory: GameFactoryTrait> EventHandlerTrait for TcpOutput<GameFactory> {
    type Event = ();
    type ThreadReturn = ();

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> EventHandleResult<Self> {
        match channel_event {
            ReceivedEvent(_, ()) => Continue(TryForNextEvent(self)),
            Timeout => Continue(WaitForNextEvent(self)),
            ChannelEmpty => Continue(WaitForNextEvent(self)),
            ChannelDisconnected => Break(())
        }
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn { () }
}