use crate::game_time::{
    FrameDuration,
    GameTimerScheduler,
};
use crate::gamemanager::{
    Manager,
    ManagerEvent,
};
use crate::interface::{
    GameTrait,
    InitialInformation,
    RenderReceiverMessage,
};
use crate::messaging::InputMessage;
use crate::server::clientaddress::ClientAddress;
use crate::server::servergametimerobserver::ServerGameTimerObserver;
use crate::server::servermanagerobserver::ServerManagerObserver;
use crate::server::tcpinput::TcpInput;
use crate::server::tcpoutput::TcpOutputEvent::SendInitialInformation;
use crate::server::tcpoutput::{
    TcpOutput,
    TcpOutputEvent,
};
use crate::server::udphandler::UdpHandler;
use crate::server::udpinput::UdpInput;
use crate::server::udpoutput::{
    UdpOutput,
    UdpOutputEvent,
};
use crate::server::{
    ServerConfig,
    TcpConnectionHandler,
};
use crate::FrameIndex;
use commons::real_time::net::tcp::{
    TcpListenerBuilder,
    TcpReadHandlerBuilder,
    TcpReader,
    TcpStream,
};
use commons::real_time::net::udp::UdpReadHandlerBuilder;
use commons::real_time::timer_service::{
    IdleTimerService,
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

pub enum ServerCoreEvent<Game: GameTrait> {
    //TODO: create render receiver sender before spawning event handler
    StartGameEvent,
    TcpConnectionEvent(TcpStream, TcpReader),
    GameTimerTick,
    InputMessage(InputMessage<Game>),
}

pub struct ServerCore<Game: GameTrait> {
    factory: Factory,
    sender: EventSender<ServerCoreEvent<Game>>,
    render_receiver_sender: Sender<RenderReceiverMessage<Game>>,
    frame_duration: FrameDuration,
    tcp_listener_sender: EventHandlerStopper,
    tcp_inputs: Vec<EventHandlerStopper>,
    tcp_outputs: Vec<EventSender<TcpOutputEvent<Game>>>,
    input_grace_period_frames: usize,
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
    _timer_service: TimerService<(), ServerGameTimerObserver<Game>>,
    game_timer: GameTimerScheduler,
    _udp_input_sender: EventHandlerStopper,
    udp_output_senders: Vec<EventSender<UdpOutputEvent<Game>>>,
    manager_sender: EventSender<ManagerEvent<Game>>,
}

impl<Game: GameTrait> HandleEvent for ServerCore<Game> {
    type Event = ServerCoreEvent<Game>;
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, event: Self::Event) -> EventHandleResult {
        match event {
            ServerCoreEvent::StartGameEvent => {
                self.start_game()
            }
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

impl<Game: GameTrait> ServerCore<Game> {
    pub fn new(
        factory: Factory,
        sender: EventSender<ServerCoreEvent<Game>>,
        render_receiver_sender: Sender<RenderReceiverMessage<Game>>,
    ) -> Result<Self, Error> {
        let frame_duration = FrameDuration::new(Game::STEP_PERIOD);

        let udp_handler = UdpHandler::<Game>::new(factory.get_time_source().clone());

        let input_grace_period_frames = frame_duration.to_frame_count(&Game::GRACE_PERIOD) as usize;

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
            TcpConnectionHandler::<Game>::new(sender.clone()),
        )?;

        Ok(Self {
            factory,
            sender,
            render_receiver_sender,
            frame_duration,
            tcp_listener_sender,
            tcp_inputs: Vec::new(),
            tcp_outputs: Vec::new(),
            input_grace_period_frames,
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
            let result = UdpOutput::<Game>::new(
                self.factory.get_time_source().clone(),
                player_index,
                &udp_socket,
            );

            let udp_output = match result {
                Ok(udp_output) => udp_output,
                Err(err) => {
                    error!("Failed to create UdpOutput: {:?}", err);
                    return EventHandleResult::StopThread;
                }
            };

            let result = EventHandlerBuilder::new_thread(
                &self.factory,
                "ServerUdpOutput".to_string(),
                udp_output,
            );

            let udp_output_sender = match result {
                Ok(udp_output_sender) => udp_output_sender,
                Err(err) => {
                    error!("Failed to spawn UdpOutput thread: {:?}", err);
                    return EventHandleResult::StopThread;
                }
            };

            udp_outputs.push(udp_output_sender);
        }

        let udp_input = UdpInput::<Game>::new(
            self.factory.get_time_source().clone(),
            self.sender.clone(),
            listening_core.udp_handler,
            udp_outputs.clone(),
        );

        let udp_input_spawn_result = UdpReadHandlerBuilder::new_thread(
            &self.factory,
            "ServerUdpInput".to_string(),
            udp_socket.try_clone().unwrap(),
            udp_input,
        );

        let udp_input_sender = match udp_input_spawn_result {
            Ok(udp_input_sender) => udp_input_sender,
            Err(error) => {
                error!("{:?}", error);
                return EventHandleResult::StopThread;
            }
        };

        let initial_state = Game::get_initial_state(self.tcp_outputs.len());

        let server_manager_observer =
            ServerManagerObserver::<Game>::new(udp_outputs.clone(), self.render_receiver_sender.clone());

        let manager_builder =
            EventHandlerBuilder::<Manager<ServerManagerObserver<Game>>>::new(&self.factory);

        let mut idle_timer_service = IdleTimerService::new();

        let mut game_timer = GameTimerScheduler::server_new(
            self.factory.get_time_source().clone(),
            &mut idle_timer_service,
            self.frame_duration,
            ServerGameTimerObserver::new(self.sender.clone()),
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

        let server_config = ServerConfig::new(game_timer.get_start_time(), self.frame_duration);

        let server_initial_information = InitialInformation::<Game>::new(
            server_config.clone(),
            self.tcp_outputs.len(),
            usize::MAX,
            initial_state.clone(),
        );

        let send_result = self.render_receiver_sender.send(RenderReceiverMessage::InitialInformation(
            server_initial_information.clone(),
        ));

        if send_result.is_err() {
            warn!("Failed to send InitialInformation to Render Receiver");
            return EventHandleResult::StopThread;
        }

        if self.render_receiver_sender
            .send(RenderReceiverMessage::StartTime(start_time))
            .is_err()
        {
            warn!("Failed to send StartTime to Render Receiver");
            return EventHandleResult::StopThread;
        }

        for tcp_output in self.tcp_outputs.iter() {
            let send_result = tcp_output.send_event(SendInitialInformation(
                server_config.clone(),
                self.tcp_outputs.len(),
                initial_state.clone(),
            ));

            if send_result.is_err() {
                warn!("Failed to send InitialInformation to TcpOutput");
                return EventHandleResult::StopThread;
            }
        }

        if manager_builder
            .get_sender()
            .send_event(ManagerEvent::DropStepsBeforeEvent(frame_index.usize()))
            .is_err()
        {
            warn!("Failed to send DropSteps to Game Manager");
            return EventHandleResult::StopThread;
        }

        let manager_sender = manager_builder
            .spawn_thread(
                "ServerManager".to_string(),
                Manager::new(
                    self.factory.get_time_source().clone(),
                    server_manager_observer,
                    server_initial_information.clone(),
                ),
            )
            .unwrap();

        self.state = State::Running(RunningCore {
            _timer_service: timer_service,
            game_timer,
            _udp_input_sender: udp_input_sender,
            udp_output_senders: udp_outputs,
            manager_sender,
        });

        return self.send_new_frame_index(FrameIndex::zero());
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
        tcp_receiver: TcpReader,
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

        let tcp_input_join_handle = TcpReadHandlerBuilder::new_thread(
            &self.factory,
            "ServerTcpInput".to_string(),
            tcp_receiver,
            TcpInput::new(),
        )
        .unwrap();

        self.tcp_inputs.push(tcp_input_join_handle);

        listening_core.udp_handler.on_client_address(client_address);

        let tcp_output_sender = EventHandlerBuilder::new_thread(
            &self.factory,
            "ServerTcpOutput".to_string(),
            TcpOutput::<Game>::new(player_index, tcp_stream),
        )
        .unwrap();

        self.tcp_outputs.push(tcp_output_sender);

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

        let drop_steps_before = if frame_index.usize() > self.input_grace_period_frames {
            frame_index - self.input_grace_period_frames
        } else {
            FrameIndex::zero()
        };

        //the manager needs its lowest step to not have any new inputs
        if drop_steps_before.usize() > 1 {
            let send_result =
                running_core
                    .manager_sender
                    .send_event(ManagerEvent::DropStepsBeforeEvent(
                        drop_steps_before.usize() - 1,
                    ));

            if send_result.is_err() {
                warn!("Failed to send DropSteps to Game Manager");
                return EventHandleResult::StopThread;
            }
        }

        let send_result = running_core
            .manager_sender
            .send_event(ManagerEvent::SetRequestedStepEvent(frame_index.usize() + 1));

        if send_result.is_err() {
            warn!("Failed to send RequestedStep to Game Manager");
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
    fn on_input_message(&self, input_message: InputMessage<Game>) -> EventHandleResult {
        //TODO: is game started?

        let running_core = match &self.state {
            State::Running(running_core) => running_core,
            _ => {
                warn!("ServerCore is not running");
                return EventHandleResult::TryForNextEvent;
            }
        };

        let current_frame_index = running_core.game_timer.get_current_frame_index();
        let drop_steps_before = if current_frame_index.usize() > self.input_grace_period_frames {
            current_frame_index - self.input_grace_period_frames
        } else {
            FrameIndex::zero()
        };

        if drop_steps_before <= input_message.get_frame_index() {
            let send_result = running_core
                .manager_sender
                .send_event(ManagerEvent::InputEvent(input_message.clone()));

            if send_result.is_err() {
                warn!("Failed to send InputEvent to Game Manager");
                return EventHandleResult::StopThread;
            }

            for udp_output in running_core.udp_output_senders.iter() {
                let send_result =
                    udp_output.send_event(UdpOutputEvent::SendInputMessage(input_message.clone()));

                if send_result.is_err() {
                    warn!("Failed to send InputEvent to UdpOutput");
                    return EventHandleResult::StopThread;
                }
            }
        }

        return EventHandleResult::TryForNextEvent;
    }
}
