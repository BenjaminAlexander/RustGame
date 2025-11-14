use crate::interface::{
    GameTrait,
    InitialInformation,
};
use crate::messaging::ToClientMessageTCP;
use crate::server::tcpoutput::TcpOutputEvent::SendInitialInformation;
use crate::server::ServerConfig;
use commons::real_time::net::tcp::TcpStream;
use commons::real_time::{EventHandleResult, HandleEvent, ReceiveMetaData};
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
        &mut self,
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

        return EventHandleResult::TryForNextEvent;
    }
}

impl<Game: GameTrait> HandleEvent for TcpOutput<Game> {
    type Event = TcpOutputEvent<Game>;
    type ThreadReturn = ();

    fn on_stop(self, _: ReceiveMetaData) -> Self::ThreadReturn {
        ()
    }
    
    fn on_event(&mut self, _: ReceiveMetaData, event: Self::Event) -> EventHandleResult<Self> {
        match event {
            SendInitialInformation(server_config, player_count, initial_state) => self.send_initial_information(server_config, player_count, initial_state),
        }
    }
    
    fn on_channel_disconnect(&mut self) -> EventHandleResult<Self> {
        EventHandleResult::StopThread(())
    }
}
