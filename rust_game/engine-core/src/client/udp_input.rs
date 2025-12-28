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
use commons::real_time::{EventHandlerStopper, Factory, TimeSource};
use commons::real_time::net::udp::{HandleUdpRead, UdpReadHandlerBuilder, UdpSocket};
use log::{
    error,
    warn,
};
use std::io::Error;
use std::net::SocketAddr;
use std::ops::ControlFlow;

pub struct UdpInput {
    stopper: EventHandlerStopper
}

impl UdpInput {
    pub fn new<Game: Play>(
        factory: &Factory,
        udp_socket: UdpSocket,
        client_core: ClientCore<Game>,
        frame_manager: FrameManager<Game>,
    ) -> Result<Self, Error> {

        let time_source = factory.get_time_source();

        let udp_read_handler = UdpReadHandler {
            fragment_assembler: FragmentAssembler::new(
                time_source.clone(), 
                //TODO: make this more configurable
                5
            ),
            client_core,
            frame_manager,
            time_source: time_source.clone(),
        };

        let stopper = UdpReadHandlerBuilder::new_thread(
            factory,
            "ClientUdpInput".to_string(),
            udp_socket,
            udp_read_handler,
        )?;

        Ok(Self{stopper})
    }

    pub fn stop(&self) -> Result<(), ()> {
        self.stopper.send_stop_thread()
    }
}

struct UdpReadHandler<Game: Play> {
    time_source: TimeSource,
    fragment_assembler: FragmentAssembler,
    client_core: ClientCore<Game>,
    frame_manager: FrameManager<Game>,
}

impl<Game: Play> UdpReadHandler<Game> {
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

impl<Game: Play> HandleUdpRead for UdpReadHandler<Game> {
    //TODO: check peer address against server address
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
