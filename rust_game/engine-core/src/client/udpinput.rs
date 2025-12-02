use crate::client::ClientCoreEvent;
use crate::game_time::{
    CompletedPing,
    PingResponse,
};
use crate::gamemanager::ManagerEvent;
use crate::messaging::{
    FragmentAssembler,
    MessageFragment,
    UdpToClientMessage,
};
use crate::GameTrait;
use commons::real_time::net::udp::HandleUdpRead;
use commons::real_time::{
    EventSender,
    TimeSource,
};
use log::{
    error,
    warn,
};
use std::io;
use std::net::SocketAddr;
use std::ops::ControlFlow;

pub struct UdpInput<Game: GameTrait> {
    time_source: TimeSource,
    fragment_assembler: FragmentAssembler,
    core_sender: EventSender<ClientCoreEvent<Game>>,
    manager_sender: EventSender<ManagerEvent<Game>>,
}

impl<Game: GameTrait> UdpInput<Game> {
    pub fn new(
        time_source: TimeSource,
        core_sender: EventSender<ClientCoreEvent<Game>>,
        manager_sender: EventSender<ManagerEvent<Game>>,
    ) -> io::Result<Self> {
        return Ok(Self {
            //TODO: make this more configurable
            fragment_assembler: FragmentAssembler::new(time_source.clone(), 5),
            core_sender,
            manager_sender,
            time_source,
        });
    }

    fn on_ping_response(&mut self, ping_response: PingResponse) -> ControlFlow<()> {
        let completed_ping = CompletedPing::new(ping_response, self.time_source.now());
        match self
            .core_sender
            .send_event(ClientCoreEvent::CompletedPing(completed_ping))
        {
            Ok(()) => ControlFlow::Continue(()),
            Err(_) => {
                error!("Error sending completed ping");
                ControlFlow::Break(())
            }
        }
    }

    fn handle_received_message(&mut self, value: UdpToClientMessage<Game>) -> ControlFlow<()> {
        match value {
            UdpToClientMessage::InputMessage(input_message) => {

                let frame_index = input_message.get_frame_index();
                let player_index = input_message.get_player_index();

                let event = match input_message.take_input() {
                    Some(input) => ManagerEvent::InputEvent { 
                        frame_index, 
                        player_index, 
                        input, 
                        is_authoritative: true 
                    },
                    None => ManagerEvent::AuthoritativeMissingInputEvent { frame_index, player_index },
                };

                let send_result = self
                    .manager_sender
                    .send_event(event);

                if send_result.is_err() {
                    warn!("Failed to send InputEvent to Game Manager");
                    return ControlFlow::Break(());
                }
            }
            UdpToClientMessage::StateMessage(state_message) => {

                let send_result = self
                    .manager_sender
                    .send_event(ManagerEvent::StateEvent(state_message));

                if send_result.is_err() {
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

impl<Game: GameTrait> HandleUdpRead for UdpInput<Game> {
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
