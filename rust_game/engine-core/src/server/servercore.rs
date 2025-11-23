use self::ServerCoreEvent::{
    GameTimerTick,
    StartGameEvent,
    StartListenerEvent,
    TcpConnectionEvent,
};
use crate::gamemanager::{
    Manager,
    ManagerEvent,
};
use crate::gametime::{
    GameTimer,
    FrameDuration,
};
use crate::interface::{
    GameTrait,
    InitialInformation,
    RenderReceiverMessage,
};
use crate::messaging::InputMessage;
use crate::server::clientaddress::ClientAddress;
use crate::server::remoteudppeer::RemoteUdpPeer;
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
use crate::server::ServerCoreEvent::UdpPacket;
use crate::server::{
    ServerConfig,
    TcpConnectionHandler,
};
use commons::real_time::net::tcp::{
    TcpListenerBuilder,
    TcpReadHandlerBuilder,
    TcpReader,
    TcpStream,
};
use commons::real_time::net::udp::{
    UdpReadHandlerBuilder,
    UdpSocket,
};
use commons::real_time::net::MAX_UDP_DATAGRAM_SIZE;
use commons::real_time::timer_service::{IdleTimerService, TimerService};
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
use std::net::{
    Ipv4Addr,
    SocketAddr,
    SocketAddrV4,
};
use std::ops::Sub;
use std::str::FromStr;

pub enum ServerCoreEvent<Game: GameTrait> {
    //TODO: start listener before spawning event handler
    StartListenerEvent,

    //TODO: create render receiver sender before spawning event handler
    StartGameEvent(Sender<RenderReceiverMessage<Game>>),
    TcpConnectionEvent(TcpStream, TcpReader),
    GameTimerTick,
    UdpPacket(SocketAddr, usize, [u8; MAX_UDP_DATAGRAM_SIZE]),
}

pub struct ServerCore<Game: GameTrait> {
    factory: Factory,
    sender: EventSender<ServerCoreEvent<Game>>,
    game_is_started: bool,
    server_config: ServerConfig,
    tcp_listener_sender_option: Option<EventHandlerStopper>,
    timer_service: TimerService<(), ServerGameTimerObserver<Game>>,
    game_timer: GameTimer,
    tcp_inputs: Vec<EventHandlerStopper>,
    tcp_outputs: Vec<EventSender<TcpOutputEvent<Game>>>,
    udp_socket: Option<UdpSocket>,
    udp_outputs: Vec<EventSender<UdpOutputEvent<Game>>>,
    udp_input_sender_option: Option<EventHandlerStopper>,
    udp_handler: UdpHandler<Game>,
    manager_sender_option: Option<EventSender<ManagerEvent<Game>>>,
    render_receiver_sender: Option<Sender<RenderReceiverMessage<Game>>>,
    drop_steps_before: usize,
}

impl<Game: GameTrait> HandleEvent for ServerCore<Game> {
    type Event = ServerCoreEvent<Game>;
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, event: Self::Event) -> EventHandleResult {
        match event {
            StartListenerEvent => self.start_listener(),
            StartGameEvent(render_receiver_sender) => self.start_game(render_receiver_sender),
            TcpConnectionEvent(tcp_stream, tcp_reader) => {
                self.on_tcp_connection(tcp_stream, tcp_reader)
            }
            GameTimerTick => self.on_game_timer_tick(),
            UdpPacket(source, len, buf) => self.on_udp_packet(source, len, buf),
        }
    }

    fn on_stop_self(self) -> Self::ThreadReturn {
        ()
    }
}

impl<Game: GameTrait> ServerCore<Game> {
    pub fn new(factory: Factory, sender: EventSender<ServerCoreEvent<Game>>) -> Result<Self, Error> {
        let game_timer_config = FrameDuration::new(Game::STEP_PERIOD);
        let server_config = ServerConfig::new(game_timer_config);

        let udp_handler = UdpHandler::<Game>::new(factory.get_time_source().clone());

        let mut idle_timer_service = IdleTimerService::new();

        let game_timer = GameTimer::new(
            &factory,
            &mut idle_timer_service,
            *server_config.get_game_timer_config(),
            0,
            ServerGameTimerObserver::new(sender.clone()),
        );

        let timer_service = idle_timer_service.start(&factory)?;

        Ok(Self {
            factory,
            sender,
            game_is_started: false,
            server_config,
            tcp_listener_sender_option: None,
            timer_service,
            game_timer,
            tcp_inputs: Vec::new(),
            tcp_outputs: Vec::new(),
            udp_outputs: Vec::new(),
            drop_steps_before: 0,
            udp_socket: None,
            udp_input_sender_option: None,
            udp_handler,
            manager_sender_option: None,
            render_receiver_sender: None,
        })
    }

    fn start_listener(&mut self) -> EventHandleResult {
        //TODO: on error, make sure other threads are closed
        //TODO: could these other threads be started somewhere else?

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

        let udp_input = UdpInput::<Game>::new(self.sender.clone());

        let udp_input_builder = UdpReadHandlerBuilder::new_thread(
            &self.factory,
            "ServerUdpInput".to_string(),
            udp_socket.try_clone().unwrap(),
            udp_input,
        );

        self.udp_socket = Some(udp_socket);

        self.udp_input_sender_option = Some(match udp_input_builder {
            Ok(udp_input_sender) => udp_input_sender,
            Err(error) => {
                error!("{:?}", error);
                return EventHandleResult::StopThread;
            }
        });

        //Bind to TcpListener Socket
        let socket_addr_v4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), Game::TCP_PORT);
        let socket_addr = SocketAddr::from(socket_addr_v4);

        let tcp_listener_sender_result = TcpListenerBuilder::new_thread(
            &self.factory,
            "ServerTcpListener".to_string(),
            socket_addr,
            TcpConnectionHandler::<Game>::new(self.sender.clone()),
        );

        match tcp_listener_sender_result {
            Ok(tcp_listener_sender) => {
                self.tcp_listener_sender_option = Some(tcp_listener_sender);
                return EventHandleResult::TryForNextEvent;
            }
            Err(error) => {
                error!("Error starting Tcp Listener Thread: {:?}", error);
                return EventHandleResult::StopThread;
            }
        }
    }

    pub(super) fn on_remote_udp_peer(&self, remote_udp_peer: RemoteUdpPeer) -> Result<(), ()> {
        if let Some(udp_output_sender) = self.udp_outputs.get(remote_udp_peer.get_player_index()) {
            let send_result =
                udp_output_sender.send_event(UdpOutputEvent::RemotePeer(remote_udp_peer));

            if send_result.is_err() {
                warn!("Failed to send RemotePeer to UdpOutput");
                return Err(());
            }
        }

        return Ok(());
    }

    fn start_game(
        &mut self,
        render_receiver_sender: Sender<RenderReceiverMessage<Game>>,
    ) -> EventHandleResult {
        //TODO: remove this line
        //let (render_receiver_sender, render_receiver) = RenderReceiver::<Game>::new();

        if !self.game_is_started {
            self.game_is_started = true;

            let initial_state = Game::get_initial_state(self.tcp_outputs.len());

            let mut udp_output_senders = Vec::<EventSender<UdpOutputEvent<Game>>>::new();

            for udp_output_sender in self.udp_outputs.iter() {
                udp_output_senders.push(udp_output_sender.clone());
            }

            let server_manager_observer = ServerManagerObserver::<Game>::new(
                udp_output_senders.clone(),
                render_receiver_sender.clone(),
            );

            let manager_builder =
                EventHandlerBuilder::<Manager<ServerManagerObserver<Game>>>::new(&self.factory);

            let send_result = manager_builder
                .get_sender()
                .send_event(ManagerEvent::DropStepsBeforeEvent(self.drop_steps_before));

            if send_result.is_err() {
                warn!("Failed to send DropSteps to Game Manager");
                return EventHandleResult::StopThread;
            }

            let server_initial_information = InitialInformation::<Game>::new(
                self.server_config.clone(),
                self.tcp_outputs.len(),
                usize::MAX,
                initial_state.clone(),
            );

            if send_result.is_err() {
                warn!("Failed to send InitialInformation to Game Manager");
                return EventHandleResult::StopThread;
            }

            let send_result = render_receiver_sender.send(
                RenderReceiverMessage::InitialInformation(server_initial_information.clone()),
            );

            if send_result.is_err() {
                warn!("Failed to send InitialInformation to Render Receiver");
                return EventHandleResult::StopThread;
            }

            for tcp_output in self.tcp_outputs.iter() {
                let send_result = tcp_output.send_event(SendInitialInformation(
                    self.server_config.clone(),
                    self.tcp_outputs.len(),
                    initial_state.clone(),
                ));

                if send_result.is_err() {
                    warn!("Failed to send InitialInformation to TcpOutput");
                    return EventHandleResult::StopThread;
                }
            }

            if self.game_timer.start_ticking(&self.timer_service).is_err() {
                warn!("Failed to Start the GameTimer");
                return EventHandleResult::StopThread;
            };

            self.manager_sender_option = Some(
                manager_builder
                    .spawn_thread(
                        "ServerManager".to_string(),
                        Manager::new(
                            self.factory.get_time_source().clone(),
                            server_manager_observer,
                            server_initial_information.clone(),
                        ),
                    )
                    .unwrap(),
            );
            self.render_receiver_sender = Some(render_receiver_sender);
        }

        return EventHandleResult::TryForNextEvent;
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
        if !self.game_is_started {
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

            let udp_out_sender = EventHandlerBuilder::new_thread(
                &self.factory,
                "ServerUdpOutput".to_string(),
                UdpOutput::<Game>::new(
                    self.factory.get_time_source().clone(),
                    player_index,
                    self.udp_socket.as_ref().unwrap(),
                )
                .unwrap(),
            )
            .unwrap();

            self.udp_handler.on_client_address(client_address);

            let tcp_output_sender = EventHandlerBuilder::new_thread(
                &self.factory,
                "ServerTcpOutput".to_string(),
                TcpOutput::<Game>::new(player_index, tcp_stream),
            )
            .unwrap();

            self.tcp_outputs.push(tcp_output_sender);
            self.udp_outputs.push(udp_out_sender);
        } else {
            info!(
                "TcpStream connected after the core has stated and will be dropped. {:?}",
                tcp_stream.get_peer_addr()
            );
        }

        return EventHandleResult::TryForNextEvent;
    }

    fn on_udp_packet(
        &mut self,
        source: SocketAddr,
        len: usize,
        buf: [u8; MAX_UDP_DATAGRAM_SIZE],
    ) -> EventHandleResult {
        let (remote_peer, input_message) = self.udp_handler.on_udp_packet(len, buf, source);

        //TODO: does this happen too often?  Should the core keep a list of known peers and check against that?
        //TODO: validate that inputs are coming from the right peers
        if let Some(remote_peer) = remote_peer {
            if self.on_remote_udp_peer(remote_peer).is_err() {
                return EventHandleResult::StopThread;
            }
        }

        if let Some(input_message) = input_message {
            if self.on_input_message(input_message).is_err() {
                return EventHandleResult::StopThread;
            }
        }

        return EventHandleResult::TryForNextEvent;
    }

    fn on_game_timer_tick(&mut self) -> EventHandleResult {
        let time_message = match self.game_timer.create_timer_message() {
            Some(time_message) => time_message,
            None => return EventHandleResult::TryForNextEvent,
        };

        /*
        TODO: time is also sent directly fomr gametime observer, seems like this is a bug

        for udp_output in self.udp_outputs.iter() {
            udp_output.send_event(UdpOutputEvent::SendTimeMessage(time_message.clone()));
        }
        */

        self.drop_steps_before = time_message
            .get_step_from_actual_time(time_message.get_scheduled_time().sub(&Game::GRACE_PERIOD))
            .ceil() as usize;

        if let Some(manager_sender) = self.manager_sender_option.as_ref() {
            //the manager needs its lowest step to not have any new inputs
            if self.drop_steps_before > 1 {
                let send_result = manager_sender.send_event(ManagerEvent::DropStepsBeforeEvent(
                    self.drop_steps_before - 1,
                ));

                if send_result.is_err() {
                    warn!("Failed to send DropSteps to Game Manager");
                    return EventHandleResult::StopThread;
                }
            }

            let send_result = manager_sender.send_event(ManagerEvent::SetRequestedStepEvent(
                time_message.get_step() + 1,
            ));

            if send_result.is_err() {
                warn!("Failed to send RequestedStep to Game Manager");
                return EventHandleResult::StopThread;
            }
        }

        for udp_output in self.udp_outputs.iter() {
            let send_result =
                udp_output.send_event(UdpOutputEvent::SendTimeMessage(time_message.clone()));

            if send_result.is_err() {
                warn!("Failed to send TimeMessage to UdpOutput");
                return EventHandleResult::StopThread;
            }
        }

        let send_result = self
            .render_receiver_sender
            .as_ref()
            .unwrap()
            .send(RenderReceiverMessage::TimeMessage(time_message.clone()));

        if send_result.is_err() {
            warn!("Failed to send TimeMessage to Render Receiver");
            return EventHandleResult::StopThread;
        }

        return EventHandleResult::TryForNextEvent;
    }

    pub(super) fn on_input_message(&self, input_message: InputMessage<Game>) -> Result<(), ()> {
        //TODO: is game started?

        if self.drop_steps_before <= input_message.get_step().usize()
            && self.manager_sender_option.is_some()
        {
            let send_result = self
                .manager_sender_option
                .as_ref()
                .unwrap()
                .send_event(ManagerEvent::InputEvent(input_message.clone()));

            if send_result.is_err() {
                warn!("Failed to send InputEvent to Game Manager");
                return Err(());
            }

            for udp_output in self.udp_outputs.iter() {
                let send_result =
                    udp_output.send_event(UdpOutputEvent::SendInputMessage(input_message.clone()));

                if send_result.is_err() {
                    warn!("Failed to send InputEvent to UdpOutput");
                    return Err(());
                }
            }
        }

        return Ok(());
    }
}
