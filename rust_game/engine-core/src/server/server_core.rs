use {
    crate::{
        FrameIndex,
        Play,
        frame_manager::{
            FrameManager,
            PresentInput,
        },
        game_time::GameTimerScheduler,
        interface::InitialInformation,
        messaging::{
            ToClientInputMessage,
            ToServerInputMessage,
        },
        server::{
            ServerConfig,
            player_address::PlayerAddress,
            server_frame_observer::ServerFrameObserver,
            tcp_input::TcpInput,
            tcp_listener::TcpListener,
            tcp_output::TcpOutput,
            udp_handler::UdpHandler,
            udp_input::UdpInput,
            udp_output::UdpOutput,
        },
        state_channel::StateSender,
    },
    commons::{
        real_time::{
            EventHandleResult,
            EventHandlerBuilder,
            EventSender,
            Factory,
            HandleEvent,
            ReceiveMetaData,
            net::tcp::{
                TcpReader,
                TcpStream,
            },
            timer_service::{
                IdleTimerService,
                TimerCallBack,
                TimerService,
            },
        },
        utils::unit_error,
    },
    log::{
        debug,
        error,
        info,
        warn,
    },
    std::{
        io::Error,
        mem::take,
        net::{
            Ipv4Addr,
            SocketAddr,
            SocketAddrV4,
        },
        str::FromStr,
    },
};

#[derive(Clone)]
pub struct ServerCore<Game: Play> {
    sender: EventSender<ServerCoreEvent<Game>>,
}

impl<Game: Play> ServerCore<Game> {
    pub fn new(factory: Factory, state_sender: StateSender<Game>) -> Result<Self, Error> {
        let builder = EventHandlerBuilder::new(&factory);

        let server_core = Self {
            sender: builder.get_sender().clone(),
        };

        let event_handler = EventHandler::new(factory, server_core.clone(), state_sender.clone())?;

        builder.spawn_thread("ServerCore".to_string(), event_handler)?;

        Ok(server_core)
    }

    pub fn start_game(&self) -> Result<(), ()> {
        self.sender
            .send_event(ServerCoreEvent::StartGameEvent)
            .map_err(unit_error)
    }

    pub fn stop(&self) -> Result<(), ()> {
        self.sender.send_stop_thread()
    }

    pub fn handle_tcp_connection(
        &self,
        tcp_stream: TcpStream,
        tcp_reader: TcpReader,
    ) -> Result<(), ()> {
        self.sender
            .send_event(ServerCoreEvent::TcpConnectionEvent(tcp_stream, tcp_reader))
            .map_err(unit_error)
    }

    pub fn handle_input_message(
        &self,
        input_message: ToServerInputMessage<Game>,
    ) -> Result<(), ()> {
        self.sender
            .send_event(ServerCoreEvent::InputMessage(input_message))
            .map_err(unit_error)
    }
}

impl<Game: Play> TimerCallBack for ServerCore<Game> {
    fn tick(&mut self) {
        let send_result = self.sender.send_event(ServerCoreEvent::GameTimerTick);

        //TODO: handle without panic
        //This will probably allow changes
        //to the timer service to allow timers to reschedule or cancel themselves
        if send_result.is_err() {
            panic!("Failed to send GameTimerTick to Core");
        }
    }
}

enum ServerCoreEvent<Game: Play> {
    StartGameEvent,
    TcpConnectionEvent(TcpStream, TcpReader),
    GameTimerTick,
    InputMessage(ToServerInputMessage<Game>),
}

struct EventHandler<Game: Play> {
    factory: Factory,
    server_core: ServerCore<Game>,
    state_sender: StateSender<Game>,
    tcp_listener: TcpListener,
    tcp_inputs: Vec<TcpInput>,
    tcp_outputs: Vec<TcpOutput<Game>>,
    core_state: CoreState<Game>,
}

#[derive(Default)]
enum CoreState<Game: Play> {
    Listening(ListeningCore<Game>),
    Running(RunningCore<Game>),
    #[default]
    Default,
}

struct ListeningCore<Game: Play> {
    udp_handler: UdpHandler<Game>,
}

struct RunningCore<Game: Play> {
    server_config: ServerConfig,
    timer_service: TimerService<(), ServerCore<Game>>,
    game_timer: GameTimerScheduler,
    udp_input: UdpInput,
    udp_outputs: Vec<UdpOutput<Game>>,
    frame_manager: FrameManager<Game>,
}

impl<Game: Play> RunningCore<Game> {
    fn stop(self) {
        if self.timer_service.stop().is_err() {
            debug!("Error when sending stop to TimerService");
        }

        if self.udp_input.stop().is_err() {
            debug!("Error when sending stop to UDP Input");
        }

        for udp_output in self.udp_outputs {
            if udp_output.stop().is_err() {
                debug!("Error when sending stop to UDP Output");
            }
        }

        if self.frame_manager.stop().is_err() {
            debug!("Error when sending stop to FrameManager");
        }
    }
}

impl<Game: Play> HandleEvent for EventHandler<Game> {
    type Event = ServerCoreEvent<Game>;
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, event: Self::Event) -> EventHandleResult {
        match event {
            ServerCoreEvent::StartGameEvent => self.start_game(),
            ServerCoreEvent::TcpConnectionEvent(tcp_stream, tcp_reader) => {
                self.on_tcp_connection(tcp_stream, tcp_reader)
            }
            ServerCoreEvent::GameTimerTick => self.on_game_timer_tick(),
            ServerCoreEvent::InputMessage(input_message) => self.on_input_message(input_message),
        }
    }

    fn on_stop_self(self) {
        if self.tcp_listener.stop().is_err() {
            debug!("Error when sending stop to TCP Listener");
        }

        for tcp_input in self.tcp_inputs {
            if tcp_input.stop().is_err() {
                debug!("Error when sending stop to TCP Input");
            }
        }

        for tcp_output in self.tcp_outputs {
            if tcp_output.stop().is_err() {
                debug!("Error when sending stop to TCP Output");
            }
        }

        if let CoreState::Running(running_core) = self.core_state {
            running_core.stop()
        }
    }
}

impl<Game: Play> EventHandler<Game> {
    pub fn new(
        factory: Factory,
        server_core: ServerCore<Game>,
        state_sender: StateSender<Game>,
    ) -> Result<Self, Error> {
        let udp_handler = UdpHandler::<Game>::new();

        let listening_core = ListeningCore { udp_handler };

        //Bind to TcpListener Socket
        let socket_addr_v4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), Game::TCP_PORT);
        let socket_addr = SocketAddr::from(socket_addr_v4);

        let tcp_listener = TcpListener::new(&factory, socket_addr, server_core.clone())?;

        Ok(Self {
            factory,
            server_core,
            state_sender,
            tcp_listener,
            tcp_inputs: Vec::new(),
            tcp_outputs: Vec::new(),
            core_state: CoreState::Listening(listening_core),
        })
    }

    fn start_game(&mut self) -> EventHandleResult {
        let listening_core = match take(&mut self.core_state) {
            CoreState::Listening(listening_core) => listening_core,
            _ => {
                warn!("The ServerCore is not in the expected state");
                return EventHandleResult::TryForNextEvent;
            }
        };

        //Start UDP
        let ip_addr_v4 = match Ipv4Addr::from_str("127.0.0.1") {
            Ok(ip_addr_v4) => ip_addr_v4,
            Err(error) => {
                error!("{:?}", error);
                return EventHandleResult::StopThread;
            }
        };

        let socket_addr = SocketAddr::V4(SocketAddrV4::new(ip_addr_v4, Game::UDP_PORT));

        let udp_socket = match self.factory.bind_udp_socket(socket_addr) {
            Ok(udp_socket) => udp_socket,
            Err(error) => {
                error!("{:?}", error);
                return EventHandleResult::StopThread;
            }
        };

        let mut udp_outputs = Vec::new();
        for player_index in 0..self.tcp_inputs.len() {
            let result = UdpOutput::new(self.factory.clone(), player_index, &udp_socket);

            match result {
                Ok(udp_output) => udp_outputs.push(udp_output),
                Err(err) => {
                    error!("Failed to create UdpOutput: {:?}", err);
                    return EventHandleResult::StopThread;
                }
            }
        }

        let result = UdpInput::new(
            &self.factory,
            self.server_core.clone(),
            &udp_socket,
            listening_core.udp_handler,
            udp_outputs.clone(),
        );

        let udp_input = match result {
            Ok(udp_input) => udp_input,
            Err(error) => {
                error!("Failed to create UdpInput: {:?}", error);
                return EventHandleResult::StopThread;
            }
        };

        let server_config = ServerConfig::new::<Game>(&self.factory);

        let initial_state = Game::get_initial_state(self.tcp_outputs.len());

        let mut idle_timer_service = IdleTimerService::new();

        let mut game_timer = GameTimerScheduler::server_new(
            self.factory.get_time_source().clone(),
            &mut idle_timer_service,
            &server_config,
            self.server_core.clone(),
        );

        let timer_service = match idle_timer_service.start(&self.factory) {
            Ok(timer_service) => timer_service,
            Err(err) => {
                warn!("Failed to Start the TimerService: {:?}", err);
                return EventHandleResult::StopThread;
            }
        };

        let (start_time, frame_index) = match game_timer.start_server_timer(&timer_service) {
            Ok(result) => result,
            Err(err) => {
                warn!("Failed to Start the GameTimer: {:?}", err);
                return EventHandleResult::StopThread;
            }
        };

        let server_initial_information = InitialInformation::<Game>::new(
            server_config.clone(),
            self.tcp_outputs.len(),
            usize::MAX,
            initial_state.clone(),
        );

        if self
            .state_sender
            .send_initial_information(server_initial_information.clone())
            .is_err()
        {
            warn!("Failed to send InitialInformation to Render Receiver");
            return EventHandleResult::StopThread;
        }

        if self.state_sender.send_start_time(start_time).is_err() {
            warn!("Failed to send StartTime to Render Receiver");
            return EventHandleResult::StopThread;
        }

        for tcp_output in self.tcp_outputs.iter() {
            let send_result = tcp_output.send_initial_information(
                server_config.clone(),
                self.tcp_outputs.len(),
                initial_state.clone(),
            );

            if send_result.is_err() {
                warn!("Failed to send InitialInformation to TcpOutput");
                return EventHandleResult::StopThread;
            }
        }

        let server_manager_observer =
            ServerFrameObserver::<Game>::new(udp_outputs.clone(), self.state_sender.clone());

        let frame_manager = FrameManager::new(
            &self.factory,
            server_manager_observer,
            server_initial_information,
        )
        .unwrap();

        self.core_state = CoreState::Running(RunningCore {
            server_config,
            timer_service,
            game_timer,
            udp_input,
            udp_outputs,
            frame_manager,
        });

        return self.send_new_frame_index(frame_index);
    }

    /*
    TODO:
    Server      Client
    Tcp Hello ->
        <- UdpHello
     */
    fn on_tcp_connection(
        &mut self,
        tcp_stream: TcpStream,
        tcp_reader: TcpReader,
    ) -> EventHandleResult {
        let listening_core = match &mut self.core_state {
            CoreState::Listening(listening_core) => listening_core,
            _ => {
                info!(
                    "TcpStream connected after the core has stated and will be dropped. {:?}",
                    tcp_stream.get_peer_addr()
                );
                return EventHandleResult::TryForNextEvent;
            }
        };

        info!("TcpStream accepted: {:?}", tcp_stream.get_peer_addr());

        let player_index = self.tcp_inputs.len();

        let client_address = PlayerAddress::new(player_index, tcp_stream.get_peer_addr().ip());
        listening_core.udp_handler.on_client_address(client_address);

        match TcpInput::new(&self.factory, player_index, tcp_reader) {
            Ok(tcp_input) => self.tcp_inputs.push(tcp_input),
            Err(err) => {
                error!("Failed to start TCP input thread: {:?}", err);
                return EventHandleResult::StopThread;
            }
        };

        match TcpOutput::new(&self.factory, player_index, tcp_stream) {
            Ok(tcp_output) => self.tcp_outputs.push(tcp_output),
            Err(err) => {
                error!("Failed to start TCP output thread: {:?}", err);
                return EventHandleResult::StopThread;
            }
        };

        return EventHandleResult::TryForNextEvent;
    }

    fn on_game_timer_tick(&mut self) -> EventHandleResult {
        let frame_index = match &mut self.core_state {
            CoreState::Running(running_core) => {
                match running_core.game_timer.try_advance_frame_index() {
                    Some(time_message) => time_message,
                    None => return EventHandleResult::TryForNextEvent,
                }
            }
            _ => {
                warn!("ServerCore is not running");
                return EventHandleResult::TryForNextEvent;
            }
        };

        return self.send_new_frame_index(frame_index);
    }

    fn send_new_frame_index(&mut self, frame_index: FrameIndex) -> EventHandleResult {
        let running_core = match &mut self.core_state {
            CoreState::Running(running_core) => running_core,
            _ => {
                warn!("ServerCore is not running");
                return EventHandleResult::TryForNextEvent;
            }
        };

        if running_core
            .frame_manager
            .server_advance_frame_index(frame_index)
            .is_err()
        {
            warn!("Failed to send DropSteps to Game Manager");
            return EventHandleResult::StopThread;
        }

        return EventHandleResult::TryForNextEvent;
    }

    //TODO: maybe change return type
    fn on_input_message(&self, input_message: ToServerInputMessage<Game>) -> EventHandleResult {
        //TODO: is game started?

        let running_core = match &self.core_state {
            CoreState::Running(running_core) => running_core,
            _ => {
                warn!("ServerCore is not running");
                return EventHandleResult::TryForNextEvent;
            }
        };

        let current_frame_index = running_core.game_timer.get_current_frame_index();
        let last_open_frame_index = running_core
            .server_config
            .get_last_open_frame_index(current_frame_index);

        let (input_coordinates, input) = input_message.take();
        if last_open_frame_index <= input_coordinates.get_frame_index() {
            let is_input_authoritative = input.is_authoritative();
            let present_input = PresentInput::ClientProvided(input);
            let send_result = running_core
                .frame_manager
                .insert_input(input_coordinates, present_input.clone());

            if send_result.is_err() {
                warn!("Failed to send InputEvent to Game Manager");
                return EventHandleResult::StopThread;
            }

            let to_client_message = ToClientInputMessage::new(input_coordinates, present_input);

            for (index, udp_output) in running_core.udp_outputs.iter().enumerate() {
                // don't send non-authoritative inputs back to the original client
                if !is_input_authoritative && index == input_coordinates.get_player_index() {
                    continue;
                }

                let send_result = udp_output.send_input_message(to_client_message.clone());

                if send_result.is_err() {
                    warn!("Failed to send InputEvent to UdpOutput");
                    return EventHandleResult::StopThread;
                }
            }
        }

        return EventHandleResult::TryForNextEvent;
    }
}
