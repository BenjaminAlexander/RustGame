use log::debug;
use std::net::TcpStream;
use std::io;
use crate::messaging::{ToClientMessageTCP, InitialInformation};
use std::io::Write;
use crate::interface::{GameFactoryTrait, GameTrait, ServerToClientTcpStream};
use std::marker::PhantomData;
use std::ops::ControlFlow::{Break, Continue};
use commons::net::TcpStreamTrait;
use crate::server::ServerConfig;
use crate::server::tcpoutput::TcpOutputEvent::SendInitialInformation;
use commons::threading::channel::ReceiveMetaData;
use commons::threading::eventhandling::{ChannelEvent, ChannelEventResult, EventHandlerTrait};
use commons::threading::eventhandling::WaitOrTryForNextEvent::{TryForNextEvent, WaitForNextEvent};

pub enum TcpOutputEvent<Game: GameTrait> {
    SendInitialInformation(ServerConfig, usize, Game::State)
}

pub struct TcpOutput<GameFactory: GameFactoryTrait> {
    player_index: usize,
    tcp_stream: ServerToClientTcpStream<GameFactory>
}

impl<GameFactory: GameFactoryTrait> TcpOutput<GameFactory> {

    pub fn new(player_index: usize,
               tcp_stream: ServerToClientTcpStream<GameFactory>) -> Self {

        return TcpOutput {
            player_index,
            tcp_stream
        };
    }

    fn send_initial_information(mut self, server_config: ServerConfig, player_count: usize, initial_state: <GameFactory::Game as GameTrait>::State) -> ChannelEventResult<Self> {

        let initial_information = InitialInformation::<GameFactory::Game>::new(
            server_config,
            player_count,
            self.player_index,
            initial_state
        );

        let message = ToClientMessageTCP::<GameFactory::Game>::InitialInformation(initial_information);
        self.tcp_stream.write(&message).unwrap();
        self.tcp_stream.flush().unwrap();

        debug!("Sent InitialInformation");

        return Continue(TryForNextEvent(self));
    }
}

impl<GameFactory: GameFactoryTrait> EventHandlerTrait for TcpOutput<GameFactory> {
    type Event = TcpOutputEvent<GameFactory::Game>;
    type ThreadReturn = ();

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> ChannelEventResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(_, SendInitialInformation(server_config, player_count, initial_state)) =>
                self.send_initial_information(server_config, player_count, initial_state),
            ChannelEvent::Timeout => Continue(WaitForNextEvent(self)),
            ChannelEvent::ChannelEmpty => Continue(WaitForNextEvent(self)),
            ChannelEvent::ChannelDisconnected => Break(())
        }
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn { () }
}