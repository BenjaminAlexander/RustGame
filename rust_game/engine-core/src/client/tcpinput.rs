use crate::client::clientcore::ClientCoreEvent;
use crate::client::ClientCoreEvent::OnInitialInformation;
use crate::interface::StateSender;
use crate::messaging::ToClientMessageTCP;
use crate::GameTrait;
use commons::real_time::net::tcp::HandleTcpRead;
use commons::real_time::{
    EventSender,
};
use log::{
    info,
    warn,
};
use std::ops::ControlFlow;
use std::ops::ControlFlow::*;

pub struct TcpInput<Game: GameTrait> {
    player_index: Option<usize>,
    client_core_sender: EventSender<ClientCoreEvent<Game>>,
    state_sender: StateSender<Game>
}

impl<Game: GameTrait> TcpInput<Game> {
    pub fn new(
        client_core_sender: EventSender<ClientCoreEvent<Game>>,
        state_sender: StateSender<Game>
    ) -> Self {
        return Self {
            player_index: None,
            client_core_sender,
            state_sender,
        };
    }
}

impl<Game: GameTrait> HandleTcpRead for TcpInput<Game> {
    type ReadType = ToClientMessageTCP<Game>;

    fn on_read(&mut self, message: Self::ReadType) -> ControlFlow<()> {
        match message {
            ToClientMessageTCP::InitialInformation(initial_information_message) => {
                info!(
                    "InitialInformation Received.  Player Index: {:?}",
                    initial_information_message.get_player_index()
                );

                self.player_index = Some(initial_information_message.get_player_index());

                let send_result = self
                    .client_core_sender
                    .send_event(OnInitialInformation(initial_information_message.clone()));

                if send_result.is_err() {
                    warn!("Failed to send InitialInformation to Core");
                    return Break(());
                }

                let send_result = self.state_sender.send_initial_information(initial_information_message);

                if send_result.is_err() {
                    warn!("Failed to send InitialInformation to Render Receiver");
                    return Break(());
                }
            }
        }

        return Continue(());
    }
}
