use crate::interface::{
    GameTrait,
    InitialInformation,
};
use crate::messaging::ToClientMessageTCP;
use crate::server::tcpoutput::TcpOutputEvent::SendInitialInformation;
use crate::server::ServerConfig;
use commons::net::TcpStream;
use commons::threading::channel::ReceiveMetaData;
use commons::threading::eventhandling::{
    ChannelEvent,
    EventHandleResult,
    EventHandlerTrait,
};
use log::debug;
use std::marker::PhantomData;

pub enum TcpOutputEvent<Game: GameTrait> {
    SendInitialInformation(ServerConfig, usize, Game::State),
}

pub struct TcpOutput<Game: GameTrait> {
    player_index: usize,
    tcp_stream: TcpStream,
    phantom: PhantomData<Game>,
}

impl<Game: GameTrait> TcpOutput<Game> {
    pub fn new(player_index: usize, tcp_stream: TcpStream) -> Self {
        return TcpOutput {
            player_index,
            tcp_stream,
            phantom: PhantomData,
        };
    }

    fn send_initial_information(
        mut self,
        server_config: ServerConfig,
        player_count: usize,
        initial_state: Game::State,
    ) -> EventHandleResult<Self> {
        let initial_information = InitialInformation::<Game>::new(
            server_config,
            player_count,
            self.player_index,
            initial_state,
        );

        let message = ToClientMessageTCP::<Game>::InitialInformation(initial_information);
        self.tcp_stream.write(&message).unwrap();
        self.tcp_stream.flush().unwrap();

        debug!("Sent InitialInformation");

        return EventHandleResult::TryForNextEvent(self);
    }
}

impl<Game: GameTrait> EventHandlerTrait for TcpOutput<Game> {
    type Event = TcpOutputEvent<Game>;
    type ThreadReturn = ();

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> EventHandleResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(
                _,
                SendInitialInformation(server_config, player_count, initial_state),
            ) => self.send_initial_information(server_config, player_count, initial_state),
            ChannelEvent::Timeout => EventHandleResult::WaitForNextEvent(self),
            ChannelEvent::ChannelEmpty => EventHandleResult::WaitForNextEvent(self),
            ChannelEvent::ChannelDisconnected => EventHandleResult::StopThread(()),
        }
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        ()
    }
}
