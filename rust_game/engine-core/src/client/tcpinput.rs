use crate::client::clientcore::ClientCoreEvent;
use crate::client::ClientCoreEvent::OnInitialInformation;
use crate::gamemanager::ManagerEvent;
use crate::interface::{EventSender, GameFactoryTrait, RenderReceiverMessage};
use crate::messaging::ToClientMessageTCP;
use commons::factory::FactoryTrait;
use commons::net::TcpReadHandlerTrait;
use commons::threading::channel::SenderTrait;
use commons::threading::eventhandling::EventSenderTrait;
use log::info;
use std::ops::ControlFlow;
use std::ops::ControlFlow::*;

pub struct TcpInput<GameFactory: GameFactoryTrait> {
    factory: GameFactory::Factory,
    player_index: Option<usize>,
    manager_sender: EventSender<GameFactory, ManagerEvent<GameFactory::Game>>,
    client_core_sender: EventSender<GameFactory, ClientCoreEvent<GameFactory>>,
    render_data_sender:
        <GameFactory::Factory as FactoryTrait>::Sender<RenderReceiverMessage<GameFactory::Game>>,
}

impl<GameFactory: GameFactoryTrait> TcpInput<GameFactory> {
    pub fn new(
        factory: GameFactory::Factory,
        manager_sender: EventSender<GameFactory, ManagerEvent<GameFactory::Game>>,
        client_core_sender: EventSender<GameFactory, ClientCoreEvent<GameFactory>>,
        render_data_sender: <GameFactory::Factory as FactoryTrait>::Sender<
            RenderReceiverMessage<GameFactory::Game>,
        >,
    ) -> Self {
        return Self {
            factory,
            player_index: None,
            manager_sender,
            client_core_sender,
            render_data_sender,
        };
    }
}

impl<GameFactory: GameFactoryTrait> TcpReadHandlerTrait for TcpInput<GameFactory> {
    type ReadType = ToClientMessageTCP<GameFactory::Game>;

    fn on_read(&mut self, message: Self::ReadType) -> ControlFlow<()> {
        match message {
            ToClientMessageTCP::InitialInformation(initial_information_message) => {
                info!(
                    "InitialInformation Received.  Player Index: {:?}",
                    initial_information_message.get_player_index()
                );

                self.player_index = Some(initial_information_message.get_player_index());
                self.manager_sender
                    .send_event(ManagerEvent::InitialInformationEvent(
                        initial_information_message.clone(),
                    ))
                    .unwrap();
                self.client_core_sender
                    .send_event(OnInitialInformation(initial_information_message.clone()))
                    .unwrap();
                self.render_data_sender
                    .send(RenderReceiverMessage::InitialInformation(
                        initial_information_message,
                    ))
                    .unwrap();
            }
        }

        return Continue(());
    }
}
