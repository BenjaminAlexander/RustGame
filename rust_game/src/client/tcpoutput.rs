use std::net::TcpStream;
use std::io;
use std::ops::ControlFlow::{Break, Continue};
use commons::threading::channel::ReceiveMetaData;
use commons::threading::eventhandling::{ChannelEvent, ChannelEventResult, EventHandlerTrait};
use commons::threading::eventhandling::ChannelEvent::{ReceivedEvent, ChannelEmpty, ChannelDisconnected};
use commons::threading::eventhandling::WaitOrTryForNextEvent::{TryForNextEvent, WaitForNextEvent};

//TODO: Send response to time messages to calculate ping
pub struct TcpOutput {
    tcp_stream: TcpStream
}

impl TcpOutput {

    pub fn new(tcp_stream: &TcpStream) -> io::Result<Self> {
        return Ok(Self{
            tcp_stream: tcp_stream.try_clone()?
        });
    }
}

impl EventHandlerTrait for TcpOutput {
    type Event = ();
    type ThreadReturn = ();

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> ChannelEventResult<Self> {
        match channel_event {
            ReceivedEvent(_, ()) => Continue(TryForNextEvent(self)),
            ChannelEmpty => Continue(WaitForNextEvent(self)),
            ChannelDisconnected => Break(())
        }
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn { () }
}