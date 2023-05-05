use crate::interface::GameFactoryTrait;
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::ops::ControlFlow::Continue;
use commons::net::{MAX_UDP_DATAGRAM_SIZE, UdpReadHandlerTrait};
use crate::server::servercore::ServerCoreEvent;
use commons::threading::eventhandling::{Sender, EventSenderTrait};

pub struct UdpInput<GameFactory: GameFactoryTrait> {
    core_sender: Sender<GameFactory::Factory, ServerCoreEvent<GameFactory>>
}

impl<GameFactory: GameFactoryTrait> UdpInput<GameFactory> {

    pub fn new(core_sender: Sender<GameFactory::Factory, ServerCoreEvent<GameFactory>>) -> Self {
        return Self {
            core_sender
        };
    }

}

impl<Game: GameFactoryTrait> UdpReadHandlerTrait for UdpInput<Game> {
    fn on_read(&mut self, peer_addr: SocketAddr, buf: &[u8]) -> ControlFlow<()> {
        let mut buf_to_send = [0; MAX_UDP_DATAGRAM_SIZE];
        buf_to_send[..buf.len()].copy_from_slice(buf);
        self.core_sender.send_event(ServerCoreEvent::UdpPacket(peer_addr, buf.len(), buf_to_send)).unwrap();
        return Continue(());
    }
}