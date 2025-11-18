use crate::real_time::net::udp::UdpReadHandlerTrait;
use crate::real_time::simulation::net::network_simulator::NetworkSimulator;
use crate::real_time::{
    EventHandleResult,
    HandleEvent,
    ReceiveMetaData,
};
use std::net::SocketAddr;
use std::ops::ControlFlow::{
    Break,
    Continue,
};

pub struct SimulatedUdpReadEventHandler<T: UdpReadHandlerTrait> {
    network_simulator: NetworkSimulator,
    socket_addr: SocketAddr,
    read_handler: T,
}
impl<T: UdpReadHandlerTrait> SimulatedUdpReadEventHandler<T> {
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

    fn read(&mut self, source: SocketAddr, buf: Vec<u8>) -> EventHandleResult {
        return match self.read_handler.on_read(source, &buf) {
            Continue(()) => EventHandleResult::TryForNextEvent,
            Break(()) => EventHandleResult::StopThread,
        };
    }
}

impl<T: UdpReadHandlerTrait> HandleEvent for SimulatedUdpReadEventHandler<T> {
    type Event = (SocketAddr, Vec<u8>);
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, event: Self::Event) -> EventHandleResult {
        let (source, buf) = event;
        self.read(source, buf)
    }

    fn on_stop_self(self) -> Self::ThreadReturn {
        self.network_simulator.remove_udp_reader(&self.socket_addr);
        return ();
    }
}
