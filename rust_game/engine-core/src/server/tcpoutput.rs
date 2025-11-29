use crate::interface::{
    GameTrait,
    InitialInformation,
};
use crate::messaging::ToClientMessageTCP;
use crate::server::tcpoutput::Event::SendInitialInformation;
use crate::server::ServerConfig;
use commons::real_time::net::tcp::TcpStream;
use commons::real_time::{
    EventHandleResult,
    EventHandlerBuilder,
    EventSender,
    Factory,
    HandleEvent,
    ReceiveMetaData,
};
use commons::utils::unit_error;
use log::debug;
use std::io::Error;
use std::marker::PhantomData;

pub struct TcpOutput<Game: GameTrait> {
    sender: EventSender<Event<Game>>,
}

impl<Game: GameTrait> TcpOutput<Game> {
    pub fn new(
        factory: &Factory,
        player_index: usize,
        tcp_stream: TcpStream,
    ) -> Result<Self, Error> {
        let sender = EventHandlerBuilder::new_thread(
            factory,
            format!("ServerTcpOutput-Player-{}", player_index),
            EventHandler::<Game>::new(player_index, tcp_stream),
        )?;

        Ok(Self { sender })
    }

    pub fn send_initial_information(
        &self,
        server_config: ServerConfig,
        player_count: usize,
        initial_state: Game::State,
    ) -> Result<(), ()> {
        let event = Event::SendInitialInformation(server_config, player_count, initial_state);

        self.sender.send_event(event).map_err(unit_error)
    }
}

enum Event<Game: GameTrait> {
    SendInitialInformation(ServerConfig, usize, Game::State),
}

struct EventHandler<Game: GameTrait> {
    player_index: usize,
    tcp_stream: TcpStream,
    phantom: PhantomData<Game>,
}

impl<Game: GameTrait> EventHandler<Game> {
    pub fn new(player_index: usize, tcp_stream: TcpStream) -> Self {
        return EventHandler {
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
    ) -> EventHandleResult {
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

impl<Game: GameTrait> HandleEvent for EventHandler<Game> {
    type Event = Event<Game>;
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, event: Self::Event) -> EventHandleResult {
        match event {
            SendInitialInformation(server_config, player_count, initial_state) => {
                self.send_initial_information(server_config, player_count, initial_state)
            }
        }
    }

    fn on_stop_self(self) -> Self::ThreadReturn {
        ()
    }
}
