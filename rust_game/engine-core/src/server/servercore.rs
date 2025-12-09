use crate::frame_manager::FrameManager;
use crate::game_time::GameTimerScheduler;
use crate::interface::{
    GameTrait,
    InitialInformation,
    RenderReceiverMessage,
};
use crate::messaging::ToServerInputMessage;
use crate::server::clientaddress::ClientAddress;
use crate::server::servermanagerobserver::ServerManagerObserver;
use crate::server::tcpinput::TcpInput;
use crate::server::tcpoutput::TcpOutput;
use crate::server::udphandler::UdpHandler;
use crate::server::udpinput::UdpInput;
use crate::server::udpoutput::UdpOutput;
use crate::server::{
    ServerConfig,
    TcpConnectionHandler,
};
use crate::FrameIndex;
use commons::real_time::net::tcp::{
    TcpListenerBuilder,
    TcpReader,
    TcpStream,
};
use commons::real_time::timer_service::{
    IdleTimerService,
    TimerCallBack,
    TimerService,
};
use commons::real_time::{
    EventHandleResult,
    EventHandlerBuilder,
    EventHandlerStopper,
    EventSender,
    Factory,
    HandleEvent,
    ReceiveMetaData,
    Sender,
};
use commons::utils::unit_error;
use log::{
    error,
    info,
    warn,
};
use std::io::Error;
use std::mem::take;
use std::net::{
    Ipv4Addr,
    SocketAddr,
    SocketAddrV4,
};
use std::str::FromStr;

#[derive(Clone)]
pub struct ServerCore<Game: GameTrait> {
    sender: EventSender<ServerCoreEvent<Game>>,
}

impl<Game: GameTrait> ServerCore<Game> {
    pub fn new(
        factory: Factory,
        render_receiver_sender: Sender<RenderReceiverMessage<Game>>,
    ) -> Result<Self, Error> {
        let builder = EventHandlerBuilder::new(&factory);

        let server_core = Self {
            sender: builder.get_sender().clone(),
        };

        let event_handler = ServerCoreEventHandler::new(
            factory,
            server_core.clone(),
            render_receiver_sender.clone(),
        )?;

        builder.spawn_thread("ServerCore".to_string(), event_handler)?;

        Ok(server_core)
    }

    pub fn start_game(&self) -> Result<(), ()> {
        self.sender
            .send_event(ServerCoreEvent::StartGameEvent)
            .map_err(unit_error)
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

impl<Game: GameTrait> TimerCallBack for ServerCore<Game> {
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

enum ServerCoreEvent<Game: GameTrait> {
    StartGameEvent,
    TcpConnectionEvent(TcpStream, TcpReader),
    GameTimerTick,
    InputMessage(ToServerInputMessage<Game>),
}

struct ServerCoreEventHandler<Game: GameTrait> {
    factory: Factory,
    server_core: ServerCore<Game>,
    render_receiver_sender: Sender<RenderReceiverMessage<Game>>,
    tcp_listener_sender: EventHandlerStopper,
    tcp_inputs: Vec<TcpInput>,
    tcp_outputs: Vec<TcpOutput<Game>>,
    state: State<Game>,
}

#[derive(Default)]
enum State<Game: GameTrait> {
    Listening(ListeningCore<Game>),
    Running(RunningCore<Game>),
    #[default]
    Default,
}

struct ListeningCore<Game: GameTrait> {
    udp_handler: UdpHandler<Game>,
}

struct RunningCore<Game: GameTrait> {
    server_config: ServerConfig,
    _timer_service: TimerService<(), ServerCore<Game>>,
    game_timer: GameTimerScheduler,
    _udp_input: UdpInput,
    udp_output_senders: Vec<UdpOutput<Game>>,
    frame_manager: FrameManager<Game>,
}

impl<Game: GameTrait> HandleEvent for ServerCoreEventHandler<Game> {
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

    fn on_stop_self(self) -> Self::ThreadReturn {
        ()
    }
}

impl<Game: GameTrait> ServerCoreEventHandler<Game> {
    pub fn new(
        factory: Factory,
        server_core: ServerCore<Game>,
        render_receiver_sender: Sender<RenderReceiverMessage<Game>>,
    ) -> Result<Self, Error> {
        let udp_handler = UdpHandler::<Game>::new(factory.get_time_source().clone());

        let listening_core = ListeningCore { udp_handler };

        //Bind to TcpListener Socket
        let socket_addr_v4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), Game::TCP_PORT);
        let socket_addr = SocketAddr::from(socket_addr_v4);

        //TODO: maybe use a builder and spawn all the threads together
        // This spawns the tcp listener thread before the core thread is spawned
        let tcp_listener_sender = TcpListenerBuilder::new_thread(
            &factory,
            "ServerTcpListener".to_string(),
            socket_addr,
            TcpConnectionHandler::<Game>::new(server_core.clone()),
        )?;

        Ok(Self {
            factory,
            server_core,
            render_receiver_sender,
            tcp_listener_sender,
            tcp_inputs: Vec::new(),
            tcp_outputs: Vec::new(),
            state: State::Listening(listening_core),
        })
    }

    fn start_game(&mut self) -> EventHandleResult {
        let listening_core = match take(&mut self.state) {
            State::Listening(listening_core) => listening_core,
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

        let send_result =
            self.render_receiver_sender
                .send(RenderReceiverMessage::InitialInformation(
                    server_initial_information.clone(),
                ));

        if send_result.is_err() {
            warn!("Failed to send InitialInformation to Render Receiver");
            return EventHandleResult::StopThread;
        }

        if self
            .render_receiver_sender
            .send(RenderReceiverMessage::StartTime(start_time))
            .is_err()
        {
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

        let server_manager_observer = ServerManagerObserver::<Game>::new(
            udp_outputs.clone(),
            self.render_receiver_sender.clone(),
        );

        let frame_manager = FrameManager::new(
            &self.factory,
            server_manager_observer,
            server_initial_information,
        )
        .unwrap();

        self.state = State::Running(RunningCore {
            server_config,
            _timer_service: timer_service,
            game_timer,
            _udp_input: udp_input,
            udp_output_senders: udp_outputs,
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
        let listening_core = match &mut self.state {
            State::Listening(listening_core) => listening_core,
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

        let client_address = ClientAddress::new(player_index, tcp_stream.get_peer_addr().ip());
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
        let frame_index = match &mut self.state {
            State::Running(running_core) => match running_core.game_timer.try_advance_frame_index()
            {
                Some(time_message) => time_message,
                None => return EventHandleResult::TryForNextEvent,
            },
            _ => {
                warn!("ServerCore is not running");
                return EventHandleResult::TryForNextEvent;
            }
        };

        return self.send_new_frame_index(frame_index);
    }

    fn send_new_frame_index(&mut self, frame_index: FrameIndex) -> EventHandleResult {
        let running_core = match &mut self.state {
            State::Running(running_core) => running_core,
            _ => {
                warn!("ServerCore is not running");
                return EventHandleResult::TryForNextEvent;
            }
        };

        if running_core
            .frame_manager
            .advance_frame_index(frame_index)
            .is_err()
        {
            warn!("Failed to send DropSteps to Game Manager");
            return EventHandleResult::StopThread;
        }

        if self
            .render_receiver_sender
            .send(RenderReceiverMessage::FrameIndex(frame_index))
            .is_err()
        {
            warn!("Failed to send FrameIndex to Render Receiver");
            return EventHandleResult::StopThread;
        }

        return EventHandleResult::TryForNextEvent;
    }

    //TODO: maybe change return type
    fn on_input_message(&self, input_message: ToServerInputMessage<Game>) -> EventHandleResult {
        //TODO: is game started?

        let running_core = match &self.state {
            State::Running(running_core) => running_core,
            _ => {
                warn!("ServerCore is not running");
                return EventHandleResult::TryForNextEvent;
            }
        };

        let current_frame_index = running_core.game_timer.get_current_frame_index();
        let last_open_frame_index = running_core
            .server_config
            .get_last_open_frame_index(current_frame_index);

        if last_open_frame_index <= input_message.get_frame_index() {
            let send_result = running_core.frame_manager.insert_input(
                input_message.get_frame_index(),
                input_message.get_player_index(),
                input_message.get_input().clone(),
                true,
            );

            if send_result.is_err() {
                warn!("Failed to send InputEvent to Game Manager");
                return EventHandleResult::StopThread;
            }

            let to_client_message = input_message.to_client_message();

            for udp_output in running_core.udp_output_senders.iter() {
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
