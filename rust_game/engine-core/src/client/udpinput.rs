use crate::client::ClientCoreEvent;
use crate::game_time::{
    CompletedPing,
    PingResponse,
};
use crate::gamemanager::ManagerEvent;
use crate::messaging::{
    FragmentAssembler,
    MessageFragment,
    ToClientMessageUDP,
    UdpToClientMessage,
};
use crate::GameTrait;
use commons::real_time::net::udp::HandleUdpRead;
use commons::real_time::{
    EventSender,
    TimeSource,
};
use commons::time::{
    TimeDuration,
    TimeValue,
};
use log::{
    debug,
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

    //metrics
    time_of_last_state_receive: TimeValue,
    time_of_last_input_receive: TimeValue,
    time_of_last_server_input_receive: TimeValue,
}

impl<Game: GameTrait> UdpInput<Game> {
    pub fn new(
        time_source: TimeSource,
        core_sender: EventSender<ClientCoreEvent<Game>>,
        manager_sender: EventSender<ManagerEvent<Game>>,
    ) -> io::Result<Self> {
        let now = time_source.now();

        return Ok(Self {
            //TODO: make this more configurable
            fragment_assembler: FragmentAssembler::new(time_source.clone(), 5),
            core_sender,
            manager_sender,

            //metrics
            time_of_last_state_receive: now,
            time_of_last_input_receive: now,
            time_of_last_server_input_receive: now,

            time_source,
        });
    }

    fn on_fragment(&mut self, buf: &[u8]) -> ControlFlow<()> {
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

    fn handle_received_message(&mut self, value: ToClientMessageUDP<Game>) -> ControlFlow<()> {
        let time_received = self.time_source.now();

        match value {
            ToClientMessageUDP::InputMessage(input_message) => {
                //TODO: ignore input messages from this player
                //info!("Input message: {:?}", input_message.get_step());
                self.time_of_last_input_receive = time_received;
                let send_result = self
                    .manager_sender
                    .send_event(ManagerEvent::InputEvent(input_message.clone()));

                if send_result.is_err() {
                    warn!("Failed to send InputEvent to Game Manager");
                    return ControlFlow::Break(());
                }
            }
            ToClientMessageUDP::ServerInputMessage(server_input_message) => {
                //info!("Server Input message: {:?}", server_input_message.get_step());
                self.time_of_last_server_input_receive = time_received;
                let send_result = self
                    .manager_sender
                    .send_event(ManagerEvent::ServerInputEvent(server_input_message));

                if send_result.is_err() {
                    warn!("Failed to send ServerInputEvent to Game Manager");
                    return ControlFlow::Break(());
                }
            }
            ToClientMessageUDP::StateMessage(state_message) => {
                //info!("State message: {:?}", state_message.get_sequence());

                let duration_since_last_state =
                    time_received.duration_since(&self.time_of_last_state_receive);
                if duration_since_last_state > TimeDuration::ONE_SECOND {
                    //TODO: this should probably be a warn
                    debug!("It has been {:?} since last state message was received. Now: {:?}, Last: {:?}",
                            duration_since_last_state, time_received, self.time_of_last_state_receive);
                }

                self.time_of_last_state_receive = time_received;
                let send_result = self
                    .manager_sender
                    .send_event(ManagerEvent::StateEvent(state_message));

                if send_result.is_err() {
                    warn!("Failed to send StateMessage to Game Manager");
                    return ControlFlow::Break(());
                }
            }
        };

        return ControlFlow::Continue(());
    }
}

impl<Game: GameTrait> HandleUdpRead for UdpInput<Game> {
    fn on_read(&mut self, peer_addr: SocketAddr, buf: &[u8]) -> ControlFlow<()> {
        let message: UdpToClientMessage = match rmp_serde::from_slice(&buf) {
            Ok(message) => message,
            Err(err) => {
                error!("Error deserializing: {:?}", err);
                return ControlFlow::Continue(());
            }
        };

        match message {
            UdpToClientMessage::PingResponse(ping_response) => self.on_ping_response(ping_response),
            UdpToClientMessage::Fragment(buf) => self.on_fragment(&buf),
        }
    }
}
