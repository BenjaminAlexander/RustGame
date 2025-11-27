use crate::game_time::PingRequest;
use crate::messaging::{
    InputMessage,
    UdpToServerMessage,
};
use crate::server::servercore::ServerCoreEvent;
use crate::server::udphandler::UdpHandler;
use crate::server::udpoutput::UdpOutputEvent;
use crate::GameTrait;
use commons::real_time::net::udp::HandleUdpRead;
use commons::real_time::{
    EventSender,
    TimeSource,
};
use log::{
    error,
    info,
    warn,
};
use std::net::SocketAddr;
use std::ops::ControlFlow;

pub struct UdpInput<Game: GameTrait> {
    time_source: TimeSource,
    core_sender: EventSender<ServerCoreEvent<Game>>,
    udp_handler: UdpHandler<Game>,
    udp_output_senders: Vec<EventSender<UdpOutputEvent<Game>>>,
}

impl<Game: GameTrait> UdpInput<Game> {
    pub fn new(
        time_source: TimeSource,
        core_sender: EventSender<ServerCoreEvent<Game>>,
        udp_handler: UdpHandler<Game>,
        udp_output_senders: Vec<EventSender<UdpOutputEvent<Game>>>,
    ) -> Self {
        return Self {
            time_source,
            core_sender,
            udp_handler,
            udp_output_senders,
        };
    }

    fn on_input_message(&mut self, input_message: InputMessage<Game>) -> ControlFlow<()> {
        let result = self
            .core_sender
            .send_event(ServerCoreEvent::InputMessage(input_message));

        match result {
            Ok(()) => ControlFlow::Continue(()),
            Err(_) => {
                warn!("Error sending InputMessage");
                ControlFlow::Break(())
            }
        }
    }

    fn on_ping_request(&mut self, ping_request: PingRequest) -> ControlFlow<()> {
        let udp_output_sender = match self.udp_output_senders.get(ping_request.get_player_index()) {
            Some(udp_output_sender) => udp_output_sender,
            None => {
                warn!(
                    "Invalid player index: {:?}",
                    ping_request.get_player_index()
                );
                return ControlFlow::Continue(());
            }
        };

        let result = udp_output_sender.send_event(UdpOutputEvent::PingRequest {
            time_received: self.time_source.now(),
            ping_request,
        });

        match result {
            Ok(()) => ControlFlow::Continue(()),
            Err(_) => {
                error!("Failed to send PingRequest to Udp Output");
                ControlFlow::Break(())
            }
        }
    }
}

impl<Game: GameTrait> HandleUdpRead for UdpInput<Game> {
    fn on_read(&mut self, peer_addr: SocketAddr, buf: &[u8]) -> ControlFlow<()> {
        let (remote_udp_peer_option, message_option) =
            self.udp_handler.on_udp_packet(buf, peer_addr);

        if let Some(message) = message_option {
            //TODO: clean up this nested if let.  If we got a message, we definitly have a remote peer
            if let Some(remote_udp_peer) = remote_udp_peer_option {
                info!("Received UDP remote peer");

                let udp_output_sender =
                    match self.udp_output_senders.get(message.get_player_index()) {
                        Some(udp_output_sender) => udp_output_sender,
                        None => {
                            warn!("Invalid player index: {:?}", message.get_player_index());
                            return ControlFlow::Continue(());
                        }
                    };

                let result =
                    udp_output_sender.send_event(UdpOutputEvent::RemotePeer(remote_udp_peer));
                if result.is_err() {
                    warn!("Error sending RemoteUdpPeer");
                    return ControlFlow::Break(());
                }
            }

            return match message {
                UdpToServerMessage::PingRequest(ping_request) => self.on_ping_request(ping_request),
                UdpToServerMessage::Input(input_message) => self.on_input_message(input_message),
            };
        }

        return ControlFlow::Continue(());
    }
}
