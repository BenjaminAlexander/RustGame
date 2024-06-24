use crate::interface::{
    EventSender,
    GameFactoryTrait,
};
use crate::server::servercore::ServerCoreEvent;
use commons::net::{
    UdpReadHandlerTrait,
    MAX_UDP_DATAGRAM_SIZE,
};
use commons::threading::eventhandling::EventSenderTrait;
use std::net::SocketAddr;
use std::ops::ControlFlow;

pub struct UdpInput<GameFactory: GameFactoryTrait> {
    core_sender: EventSender<GameFactory, ServerCoreEvent<GameFactory>>,
}

impl<GameFactory: GameFactoryTrait> UdpInput<GameFactory> {
    pub fn new(core_sender: EventSender<GameFactory, ServerCoreEvent<GameFactory>>) -> Self {
        return Self { core_sender };
    }
}

impl<Game: GameFactoryTrait> UdpReadHandlerTrait for UdpInput<Game> {
    fn on_read(&mut self, peer_addr: SocketAddr, buf: &[u8]) -> ControlFlow<()> {
        let mut buf_to_send = [0; MAX_UDP_DATAGRAM_SIZE];
        buf_to_send[..buf.len()].copy_from_slice(buf);

        let send_result = self.core_sender.send_event(ServerCoreEvent::UdpPacket(
            peer_addr,
            buf.len(),
            buf_to_send,
        ));

        return match send_result {
            Ok(_) => ControlFlow::Continue(()),
            Err(_) => ControlFlow::Break(()),
        };
    }
}
