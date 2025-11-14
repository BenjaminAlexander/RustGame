use crate::net::UdpReadHandlerTrait;
use crate::real_time::{EventHandleResult, HandleEvent, ReceiveMetaData};
use crate::single_threaded_simulator::net::NetworkSimulator;
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

impl<T: UdpReadHandlerTrait> HandleEvent for UdpReadEventHandler<T> {
    type Event = (SocketAddr, Vec<u8>);
    type ThreadReturn = ();

    fn on_stop(self, _: ReceiveMetaData) -> Self::ThreadReturn {
        self.network_simulator.remove_udp_reader(&self.socket_addr);
        return ();
    }
    
    fn on_event(&mut self, _: ReceiveMetaData, event: Self::Event) -> EventHandleResult<Self> {
        let (source, buf) = event;
        self.read(source, buf)
    }

    fn on_channel_disconnect(&mut self) -> EventHandleResult<Self> {
        EventHandleResult::StopThread(())
    }
}
