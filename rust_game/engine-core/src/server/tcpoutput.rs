use log::debug;
use crate::messaging::{ToClientMessageTCP, InitialInformation};
use crate::interface::{GameFactoryTrait, GameTrait, TcpWriter};
use commons::net::TcpWriterTrait;
use crate::server::ServerConfig;
use crate::server::tcpoutput::TcpOutputEvent::SendInitialInformation;
use commons::threading::channel::ReceiveMetaData;
use commons::threading::eventhandling::{ChannelEvent, EventHandleResult, EventHandlerTrait};

pub enum TcpOutputEvent<Game: GameTrait> {
    SendInitialInformation(ServerConfig, usize, Game::State)
}

pub struct TcpOutput<GameFactory: GameFactoryTrait> {
    player_index: usize,
    tcp_sender: TcpWriter<GameFactory>
}

impl<GameFactory: GameFactoryTrait> TcpOutput<GameFactory> {

    pub fn new(player_index: usize,
               tcp_sender: TcpWriter<GameFactory>) -> Self {

        return TcpOutput {
            player_index,
            tcp_sender
        };
    }

    fn send_initial_information(mut self, server_config: ServerConfig, player_count: usize, initial_state: <GameFactory::Game as GameTrait>::State) -> EventHandleResult<Self> {

        let initial_information = InitialInformation::<GameFactory::Game>::new(
            server_config,
            player_count,
            self.player_index,
            initial_state
        );

        let message = ToClientMessageTCP::<GameFactory::Game>::InitialInformation(initial_information);
        self.tcp_sender.write(&message).unwrap();
        self.tcp_sender.flush().unwrap();

        debug!("Sent InitialInformation");

        return EventHandleResult::TryForNextEvent(self);
    }
}

impl<GameFactory: GameFactoryTrait> EventHandlerTrait for TcpOutput<GameFactory> {
    type Event = TcpOutputEvent<GameFactory::Game>;
    type ThreadReturn = ();

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> EventHandleResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(_, SendInitialInformation(server_config, player_count, initial_state)) =>
                self.send_initial_information(server_config, player_count, initial_state),
            ChannelEvent::Timeout => EventHandleResult::WaitForNextEvent(self),
            ChannelEvent::ChannelEmpty => EventHandleResult::WaitForNextEvent(self),
            ChannelEvent::ChannelDisconnected => EventHandleResult::StopThread(())
        }
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn { () }
}