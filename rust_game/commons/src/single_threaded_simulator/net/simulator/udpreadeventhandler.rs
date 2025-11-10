use crate::net::UdpReadHandlerTrait;
use crate::single_threaded_simulator::net::NetworkSimulator;
use crate::threading::channel::ReceiveMetaData;
use crate::threading::eventhandling::{
    ChannelEvent,
    EventHandleResult,
    EventHandlerTrait,
};
use std::net::SocketAddr;
use std::ops::ControlFlow::{
    Break,
    Continue,
};

pub struct UdpReadEventHandler<T: UdpReadHandlerTrait> {
    network_simulator: NetworkSimulator,
    socket_addr: SocketAddr,
    read_handler: T,
}
impl<T: UdpReadHandlerTrait> UdpReadEventHandler<T> {
    pub fn new(
        network_simulator: NetworkSimulator,
        socket_addr: SocketAddr,
        read_handler: T,
    ) -> Self {
        return Self {
            network_simulator,
            socket_addr,
            read_handler,
        };
    }

    fn read(mut self, source: SocketAddr, buf: Vec<u8>) -> EventHandleResult<Self> {
        return match self.read_handler.on_read(source, &buf) {
            Continue(read_handler) => {
                self.read_handler = read_handler;
                EventHandleResult::TryForNextEvent(self)
            }
            Break(()) => EventHandleResult::StopThread,
        };
    }
}

impl<T: UdpReadHandlerTrait> EventHandlerTrait for UdpReadEventHandler<T> {
    type Event = (SocketAddr, Vec<u8>);

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> EventHandleResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(_, (source, buf)) => self.read(source, buf),
            ChannelEvent::Timeout => EventHandleResult::TryForNextEvent(self),
            ChannelEvent::ChannelEmpty => EventHandleResult::WaitForNextEvent(self),
            ChannelEvent::ChannelDisconnected => {
                self.read_handler.on_channel_disconnected();
                EventHandleResult::StopThread
            }
        }
    }

    fn on_stop(self, receive_meta_data: ReceiveMetaData) {
        self.network_simulator.remove_udp_reader(&self.socket_addr);
        self.read_handler.on_stop(receive_meta_data);
    }
}
