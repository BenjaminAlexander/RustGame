use crate::game_time::PingRequest;
use crate::messaging::{
    ToServerInputMessage,
    UdpToServerMessage,
};
use crate::server::udphandler::UdpHandler;
use crate::server::udpoutput::UdpOutput;
use crate::server::ServerCore;
use crate::GameTrait;
use commons::real_time::net::udp::{
    HandleUdpRead,
    UdpReadHandlerBuilder,
    UdpSocket,
};
use commons::real_time::{
    EventHandlerStopper,
    Factory,
    TimeSource,
};
use log::{
    error,
    info,
    warn,
};
use std::io::Error;
use std::net::SocketAddr;
use std::ops::ControlFlow;

pub struct UdpInput {
    _stopper: EventHandlerStopper,
}

impl UdpInput {
    pub fn new<Game: GameTrait>(
        factory: &Factory,
        server_core: ServerCore<Game>,
        udp_socket: &UdpSocket,
        udp_handler: UdpHandler<Game>,
        udp_outputs: Vec<UdpOutput<Game>>,
    ) -> Result<Self, Error> {
        let udp_input = ReadHandler::<Game>::new(
            factory.get_time_source().clone(),
            server_core,
            udp_handler,
            udp_outputs,
        );

        let stopper = UdpReadHandlerBuilder::new_thread(
            factory,
            "ServerUdpInput".to_string(),
            udp_socket.try_clone()?,
            udp_input,
        )?;

        Ok(Self { _stopper: stopper })
    }
}

struct ReadHandler<Game: GameTrait> {
    time_source: TimeSource,
    server_core: ServerCore<Game>,
    udp_handler: UdpHandler<Game>,
    udp_output_senders: Vec<UdpOutput<Game>>,
}

impl<Game: GameTrait> ReadHandler<Game> {
    pub fn new(
        time_source: TimeSource,
        server_core: ServerCore<Game>,
        udp_handler: UdpHandler<Game>,
        udp_output_senders: Vec<UdpOutput<Game>>,
    ) -> Self {
        return Self {
            time_source,
            server_core,
            udp_handler,
            udp_output_senders,
        };
    }

    fn on_input_message(&mut self, input_message: ToServerInputMessage<Game>) -> ControlFlow<()> {
        match self.server_core.handle_input_message(input_message) {
            Ok(()) => ControlFlow::Continue(()),
            Err(()) => {
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

        let result = udp_output_sender.send_ping_response(self.time_source.now(), ping_request);

        match result {
            Ok(()) => ControlFlow::Continue(()),
            Err(()) => {
                error!("Failed to send PingRequest to Udp Output");
                ControlFlow::Break(())
            }
        }
    }
}

impl<Game: GameTrait> HandleUdpRead for ReadHandler<Game> {
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

                let result = udp_output_sender.set_remote_peer(remote_udp_peer);
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
