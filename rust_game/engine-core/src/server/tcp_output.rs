use {
    crate::{
        Play,
        interface::InitialInformation,
        messaging::ToClientMessageTCP,
        server::{
            ServerConfig,
            tcp_output::Event::SendInitialInformation,
        },
    },
    commons::{
        real_time::{
            EventHandleResult,
            EventHandlerBuilder,
            EventSender,
            Factory,
            HandleEvent,
            ReceiveMetaData,
            net::tcp::TcpStream,
        },
        utils::unit_error,
    },
    log::debug,
    std::{
        io::Error,
        marker::PhantomData,
    },
};

pub struct TcpOutput<Game: Play> {
    //TODO: add stop
    sender: EventSender<Event<Game>>,
}

impl<Game: Play> TcpOutput<Game> {
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

    pub fn stop(&self) -> Result<(), ()> {
        self.sender.send_stop_thread()
    }
}

enum Event<Game: Play> {
    SendInitialInformation(ServerConfig, usize, Game::State),
}

struct EventHandler<Game: Play> {
    player_index: usize,
    tcp_stream: TcpStream,
    phantom: PhantomData<Game>,
}

impl<Game: Play> EventHandler<Game> {
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

        //TODO: flush the stream when no events are pending
        self.tcp_stream.flush().unwrap();

        debug!("Sent InitialInformation");

        return EventHandleResult::TryForNextEvent;
    }
}

impl<Game: Play> HandleEvent for EventHandler<Game> {
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
