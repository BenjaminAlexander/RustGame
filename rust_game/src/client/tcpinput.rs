use log::{error, info, warn};
use std::net::TcpStream;
use commons::threading::eventhandling;
use crate::messaging::ToClientMessageTCP;
use std::io;
use std::ops::ControlFlow;
use std::ops::ControlFlow::*;
use commons::factory::FactoryTrait;
use commons::net::{TcpReaderTrait, TcpReadHandlerTrait};
use crate::client::clientcore::ClientCoreEvent;
use crate::client::ClientCoreEvent::OnInitialInformation;
use crate::gamemanager::{ManagerEvent, RenderReceiverMessage};
use crate::interface::{GameFactoryTrait, TcpReceiver};
use commons::threading::channel::{ReceiveMetaData, SenderTrait};
use commons::threading::eventhandling::EventSenderTrait;
use commons::threading::listener::{ChannelEvent, ListenerEventResult, ListenerTrait, ListenResult};
use commons::threading::listener::ListenedOrDidNotListen::Listened;

pub struct TcpInput <GameFactory: GameFactoryTrait> {
    factory: GameFactory::Factory,
    player_index: Option<usize>,
    manager_sender: eventhandling::Sender<GameFactory::Factory, ManagerEvent<GameFactory::Game>>,
    client_core_sender: eventhandling::Sender<GameFactory::Factory, ClientCoreEvent<GameFactory>>,
    render_data_sender: <GameFactory::Factory as FactoryTrait>::Sender<RenderReceiverMessage<GameFactory::Game>>
}

impl<GameFactory: GameFactoryTrait> TcpInput<GameFactory> {

    pub fn new(
        factory: GameFactory::Factory,
        manager_sender: eventhandling::Sender<GameFactory::Factory, ManagerEvent<GameFactory::Game>>,
        client_core_sender: eventhandling::Sender<GameFactory::Factory, ClientCoreEvent<GameFactory>>,
        render_data_sender: <GameFactory::Factory as FactoryTrait>::Sender<RenderReceiverMessage<GameFactory::Game>>) -> Self {

        return Self {
            factory,
            player_index: None,
            manager_sender,
            client_core_sender,
            render_data_sender
        };
    }
}

impl <GameFactory: GameFactoryTrait> TcpReadHandlerTrait for TcpInput<GameFactory> {
    type ReadType = ToClientMessageTCP<GameFactory::Game>;

    fn on_read(&mut self, message: Self::ReadType) -> ControlFlow<()> {
        match message {
            ToClientMessageTCP::InitialInformation(initial_information_message) => {
                info!("InitialInformation Received.  Player Index: {:?}", initial_information_message.get_player_index());

                self.player_index = Some(initial_information_message.get_player_index());
                self.manager_sender.send_event(ManagerEvent::InitialInformationEvent(initial_information_message.clone())).unwrap();
                self.client_core_sender.send_event(OnInitialInformation(initial_information_message.clone())).unwrap();
                self.render_data_sender.send(RenderReceiverMessage::InitialInformation(initial_information_message)).unwrap();
            }
        }

        return Continue(());
    }
}