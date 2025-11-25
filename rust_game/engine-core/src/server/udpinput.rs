use crate::server::servercore::ServerCoreEvent;
use crate::server::udphandler::UdpHandler;
use crate::GameTrait;
use commons::real_time::net::udp::HandleUdpRead;
use commons::real_time::EventSender;
use log::warn;
use std::net::SocketAddr;
use std::ops::ControlFlow;

pub struct UdpInput<Game: GameTrait> {
    core_sender: EventSender<ServerCoreEvent<Game>>,
    udp_handler: UdpHandler<Game>,
}

impl<Game: GameTrait> UdpInput<Game> {
    pub fn new(
        core_sender: EventSender<ServerCoreEvent<Game>>,
        udp_handler: UdpHandler<Game>,
    ) -> Self {
        return Self {
            core_sender,
            udp_handler,
        };
    }
}

impl<Game: GameTrait> HandleUdpRead for UdpInput<Game> {
    fn on_read(&mut self, peer_addr: SocketAddr, buf: &[u8]) -> ControlFlow<()> {
        let (remote_udp_peer_option, input_message_option) =
            self.udp_handler.on_udp_packet(buf, peer_addr);

        if let Some(remote_udp_peer) = remote_udp_peer_option {
            let result = self
                .core_sender
                .send_event(ServerCoreEvent::RemoteUdpPeer(remote_udp_peer));
            if result.is_err() {
                warn!("Error sending RemoteUdpPeer");
                return ControlFlow::Break(());
            }
        }

        if let Some(input_message) = input_message_option {
            let result = self
                .core_sender
                .send_event(ServerCoreEvent::InputMessage(input_message));
            if result.is_err() {
                warn!("Error sending InputMessage");
                return ControlFlow::Break(());
            }
        }

        return ControlFlow::Continue(());
    }
}
