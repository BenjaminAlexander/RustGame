use crate::client::clientcore::ClientCoreEvent;
use crate::client::ClientCoreEvent::OnInitialInformation;
use crate::gamemanager::ManagerEvent;
use crate::interface::RenderReceiverMessage;
use crate::messaging::ToClientMessageTCP;
use crate::GameTrait;
use commons::real_time::net::tcp::TcpReadHandlerTrait;
use commons::real_time::{
    EventSender,
    Sender,
};
use log::{
    info,
    warn,
};
use std::ops::ControlFlow;
use std::ops::ControlFlow::*;

pub struct TcpInput<Game: GameTrait> {
    player_index: Option<usize>,
    manager_sender: EventSender<ManagerEvent<Game>>,
    client_core_sender: EventSender<ClientCoreEvent<Game>>,
    render_data_sender: Sender<RenderReceiverMessage<Game>>,
}

impl<Game: GameTrait> TcpInput<Game> {
    pub fn new(
        manager_sender: EventSender<ManagerEvent<Game>>,
        client_core_sender: EventSender<ClientCoreEvent<Game>>,
        render_data_sender: Sender<RenderReceiverMessage<Game>>,
    ) -> Self {
        return Self {
            player_index: None,
            manager_sender,
            client_core_sender,
            render_data_sender,
        };
    }
}

impl<Game: GameTrait> TcpReadHandlerTrait for TcpInput<Game> {
    type ReadType = ToClientMessageTCP<Game>;

    fn on_read(&mut self, message: Self::ReadType) -> ControlFlow<()> {
        match message {
            ToClientMessageTCP::InitialInformation(initial_information_message) => {
                info!(
                    "InitialInformation Received.  Player Index: {:?}",
                    initial_information_message.get_player_index()
                );

                self.player_index = Some(initial_information_message.get_player_index());

                let send_result =
                    self.manager_sender
                        .send_event(ManagerEvent::InitialInformationEvent(
                            initial_information_message.clone(),
                        ));

                if send_result.is_err() {
                    warn!("Failed to send InitialInformation to Game Manager");
                    return Break(());
                }

                let send_result = self
                    .client_core_sender
                    .send_event(OnInitialInformation(initial_information_message.clone()));

                if send_result.is_err() {
                    warn!("Failed to send InitialInformation to Core");
                    return Break(());
                }

                let send_result =
                    self.render_data_sender
                        .send(RenderReceiverMessage::InitialInformation(
                            initial_information_message,
                        ));

                if send_result.is_err() {
                    warn!("Failed to send InitialInformation to Render Receiver");
                    return Break(());
                }
            }
        }

        return Continue(());
    }
}
