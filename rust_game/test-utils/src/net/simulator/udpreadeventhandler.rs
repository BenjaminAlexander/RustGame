use std::net::SocketAddr;
use std::ops::ControlFlow::{Break, Continue};
use commons::net::UdpReadHandlerTrait;
use commons::threading::channel::ReceiveMetaData;
use commons::threading::eventhandling::{ChannelEvent, ChannelEventResult, EventHandlerTrait};
use commons::threading::eventhandling::WaitOrTryForNextEvent::{TryForNextEvent, WaitForNextEvent};
use crate::net::NetworkSimulator;

pub struct UdpReadEventHandler<T: UdpReadHandlerTrait> {
    network_simulator: NetworkSimulator,
    socket_addr: SocketAddr,
    read_handler: T
}
impl<T: UdpReadHandlerTrait> UdpReadEventHandler<T> {

    pub fn new(network_simulator: NetworkSimulator,  socket_addr: SocketAddr, read_handler: T) -> Self {
        return Self {
            network_simulator,
            socket_addr,
            read_handler
        };
    }

    fn read(mut self, source: SocketAddr, buf: Vec<u8>) -> ChannelEventResult<Self> {

        return match self.read_handler.on_read(source, &buf) {
            Continue(()) => Continue(TryForNextEvent(self)),
            Break(()) => Break(self.read_handler)
        };
    }
}

impl<T: UdpReadHandlerTrait> EventHandlerTrait for UdpReadEventHandler<T> {
    type Event = (SocketAddr, Vec<u8>);
    type ThreadReturn = T;

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> ChannelEventResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(_, (source, buf)) => self.read(source, buf),
            ChannelEvent::Timeout => Continue(TryForNextEvent(self)),
            ChannelEvent::ChannelEmpty => Continue(WaitForNextEvent(self)),
            ChannelEvent::ChannelDisconnected => Break(self.read_handler)
        }
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        self.network_simulator.remove_udp_reader(&self.socket_addr);
        return self.read_handler;
    }
}