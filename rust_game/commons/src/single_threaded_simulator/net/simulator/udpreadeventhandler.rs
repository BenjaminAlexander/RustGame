use crate::net::UdpReadHandlerTrait;
use crate::real_time::ReceiveMetaData;
use crate::single_threaded_simulator::net::NetworkSimulator;
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

    fn read(&mut self, source: SocketAddr, buf: Vec<u8>) -> EventHandleResult<Self> {
        return match self.read_handler.on_read(source, &buf) {
            Continue(()) => EventHandleResult::TryForNextEvent,
            Break(()) => EventHandleResult::StopThread(()),
        };
    }
}

impl<T: UdpReadHandlerTrait> EventHandlerTrait for UdpReadEventHandler<T> {
    type Event = (SocketAddr, Vec<u8>);
    type ThreadReturn = ();

    fn on_channel_event(
        &mut self,
        channel_event: ChannelEvent<Self::Event>,
    ) -> EventHandleResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(_, (source, buf)) => self.read(source, buf),
            ChannelEvent::Timeout => EventHandleResult::TryForNextEvent,
            ChannelEvent::ChannelEmpty => EventHandleResult::WaitForNextEvent,
            ChannelEvent::ChannelDisconnected => EventHandleResult::StopThread(()),
        }
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        self.network_simulator.remove_udp_reader(&self.socket_addr);
        return ();
    }
}
