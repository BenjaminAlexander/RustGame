use log::debug;
use std::net::TcpStream;
use std::io;
use crate::messaging::{ToClientMessageTCP, InitialInformation};
use std::io::Write;
use crate::interface::GameTrait;
use std::marker::PhantomData;
use std::ops::ControlFlow::{Break, Continue};
use crate::server::ServerConfig;
use crate::server::tcpoutput::TcpOutputEvent::SendInitialInformation;
use commons::threading::channel::ReceiveMetaData;
use commons::threading::eventhandling::{ChannelEvent, ChannelEventResult, EventHandlerTrait};
use commons::threading::eventhandling::WaitOrTryForNextEvent::{TryForNextEvent, WaitForNextEvent};

pub enum TcpOutputEvent<Game: GameTrait> {
    SendInitialInformation(ServerConfig, usize, Game::State)
}

pub struct TcpOutput<Game: GameTrait> {
    player_index: usize,
    tcp_stream: TcpStream,
    phantom: PhantomData<Game>
}

impl<Game: GameTrait> TcpOutput<Game> {

    pub fn new(player_index: usize,
               tcp_stream: &TcpStream) -> io::Result<Self> {

        Ok(TcpOutput{
            player_index,
            tcp_stream: tcp_stream.try_clone()?,
            phantom: PhantomData
        })
    }

    fn send_initial_information(mut self, server_config: ServerConfig, player_count: usize, initial_state: Game::State) -> ChannelEventResult<Self> {

        let initial_information = InitialInformation::<Game>::new(
            server_config,
            player_count,
            self.player_index,
            initial_state
        );

        let message = ToClientMessageTCP::<Game>::InitialInformation(initial_information);
        rmp_serde::encode::write(&mut self.tcp_stream, &message).unwrap();
        self.tcp_stream.flush().unwrap();

        debug!("Sent InitialInformation");

        return Continue(TryForNextEvent(self));
    }
}

impl<Game: GameTrait> EventHandlerTrait for TcpOutput<Game> {
    type Event = TcpOutputEvent<Game>;
    type ThreadReturn = ();

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> ChannelEventResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(_, SendInitialInformation(server_config, player_count, initial_state)) =>
                self.send_initial_information(server_config, player_count, initial_state),
            ChannelEvent::ChannelEmpty => Continue(WaitForNextEvent(self)),
            ChannelEvent::ChannelDisconnected => Break(())
        }
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn { () }
}