use crate::Play;
use crate::client::ClientCore;
use crate::frame_manager::FrameManager;
use crate::game_time::{
    CompletedPing,
    PingResponse,
};
use crate::messaging::{
    FragmentAssembler,
    MessageFragment,
    UdpToClientMessage,
};
use commons::real_time::TimeSource;
use commons::real_time::net::udp::HandleUdpRead;
use log::{
    error,
    warn,
};
use std::io;
use std::net::SocketAddr;
use std::ops::ControlFlow;

pub struct UdpInput<Game: Play> {
    time_source: TimeSource,
    fragment_assembler: FragmentAssembler,
    client_core: ClientCore<Game>,
    frame_manager: FrameManager<Game>,
}

impl<Game: Play> UdpInput<Game> {
    pub fn new(
        time_source: TimeSource,
        client_core: ClientCore<Game>,
        frame_manager: FrameManager<Game>,
    ) -> io::Result<Self> {
        return Ok(Self {
            //TODO: make this more configurable
            fragment_assembler: FragmentAssembler::new(time_source.clone(), 5),
            client_core,
            frame_manager,
            time_source,
        });
    }

    fn on_ping_response(&mut self, ping_response: PingResponse) -> ControlFlow<()> {
        let completed_ping = CompletedPing::new(ping_response, self.time_source.now());

        let result = self.client_core.handle_complete_ping(completed_ping);

        match result {
            Ok(()) => ControlFlow::Continue(()),
            Err(()) => {
                error!("Error sending completed ping");
                ControlFlow::Break(())
            }
        }
    }

    fn handle_received_message(&mut self, value: UdpToClientMessage<Game>) -> ControlFlow<()> {
        match value {
            UdpToClientMessage::InputMessage(input_message) => {
                let (input_coordinates, input) = input_message.take();

                let result = self.frame_manager.insert_input(input_coordinates, input);

                if result.is_err() {
                    warn!("Failed to send InputEvent to Game Manager");
                    return ControlFlow::Break(());
                }
            }
            UdpToClientMessage::StateMessage(state_message) => {
                let result = self
                    .frame_manager
                    .insert_state(state_message.get_frame_index(), state_message.take_state());

                if result.is_err() {
                    warn!("Failed to send StateMessage to Game Manager");
                    return ControlFlow::Break(());
                }
            }
            UdpToClientMessage::PingResponse(ping_response) => {
                return self.on_ping_response(ping_response);
            }
        };

        return ControlFlow::Continue(());
    }
}

impl<Game: Play> HandleUdpRead for UdpInput<Game> {
    fn on_read(&mut self, peer_addr: SocketAddr, buf: &[u8]) -> ControlFlow<()> {
        let fragment = MessageFragment::from_vec(buf.to_vec());

        if let Some(message_buf) = self.fragment_assembler.add_fragment(fragment) {
            match rmp_serde::from_slice(&message_buf) {
                Ok(message) => {
                    //Why does this crash the client?
                    //info!("{:?}", message);

                    return self.handle_received_message(message);
                }
                Err(error) => {
                    error!("Error: {:?}", error);
                }
            }
        }

        return ControlFlow::Continue(());
    }
}
