use crate::server::servercore::ServerCoreEvent;
use crate::GameTrait;
use commons::real_time::net::udp::UdpReadHandlerTrait;
use commons::real_time::net::MAX_UDP_DATAGRAM_SIZE;
use commons::real_time::EventSender;
use std::net::SocketAddr;
use std::ops::ControlFlow;

pub struct UdpInput<Game: GameTrait> {
    core_sender: EventSender<ServerCoreEvent<Game>>,
}

impl<Game: GameTrait> UdpInput<Game> {
    pub fn new(core_sender: EventSender<ServerCoreEvent<Game>>) -> Self {
        return Self { core_sender };
    }
}

impl<Game: GameTrait> UdpReadHandlerTrait for UdpInput<Game> {
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
