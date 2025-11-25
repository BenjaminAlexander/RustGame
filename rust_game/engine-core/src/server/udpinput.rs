use crate::messaging::UdpToServerMessage;
use crate::server::servercore::ServerCoreEvent;
use crate::server::udphandler::UdpHandler;
use crate::GameTrait;
use commons::real_time::net::udp::HandleUdpRead;
use commons::real_time::EventSender;
use log::{info, warn};
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

    fn on_fragment(&mut self, peer_addr: SocketAddr, buf: &[u8]) -> ControlFlow<()> {

        let (remote_udp_peer_option, input_message_option) =
            self.udp_handler.on_udp_packet(buf, peer_addr);

        if let Some(remote_udp_peer) = remote_udp_peer_option {

            info!("Received UDP remote peer");

            let result = self
                .core_sender
                .send_event(ServerCoreEvent::RemoteUdpPeer(remote_udp_peer));
            if result.is_err() {
                warn!("Error sending RemoteUdpPeer");
                return ControlFlow::Break(());
            }
        }

        if let Some(input_message) = input_message_option {

            info!("Received UDP input_message");

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

impl<Game: GameTrait> HandleUdpRead for UdpInput<Game> {
    fn on_read(&mut self, peer_addr: SocketAddr, buf: &[u8]) -> ControlFlow<()> {

        info!("Received UDP packet");

        let message = match rmp_serde::from_slice::<UdpToServerMessage>(buf) {
            Ok(message) => message,
            Err(err) => {
                warn!("Failed to deserialize: {:?}", err);
                return ControlFlow::Continue(());
            }
        };

        info!("Deserialized UDP packet");

        match message {
            UdpToServerMessage::PingRequest { player_index, ping_request } => todo!(),
            UdpToServerMessage::Fragment(buf) => self.on_fragment(peer_addr, &buf),
        }
    }
}
