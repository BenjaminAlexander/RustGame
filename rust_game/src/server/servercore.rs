use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket, SocketAddr};
use std::ops::ControlFlow::{Continue, Break};
use log::{error, info};
use crate::interface::{GameFactoryTrait, GameTrait, TcpReader, TcpWriter};
use crate::server::tcpinput::TcpInput;
use commons::threading::eventhandling;
use crate::server::{TcpConnectionHandler, ServerConfig};
use crate::server::tcpoutput::{TcpOutput, TcpOutputEvent};
use crate::gametime::GameTimer;
use crate::gamemanager::{Manager, ManagerEvent, RenderReceiverMessage};
use crate::messaging::{InputMessage, InitialInformation};
use std::str::FromStr;
use commons::factory::FactoryTrait;
use commons::net::{MAX_UDP_DATAGRAM_SIZE, TcpWriterTrait};
use crate::server::udpinput::UdpInput;
use crate::server::udpoutput::{UdpOutput, UdpOutputEvent};
use crate::server::clientaddress::ClientAddress;
use crate::server::remoteudppeer::RemoteUdpPeer;
use crate::server::servergametimerobserver::ServerGameTimerObserver;
use crate::server::servermanagerobserver::ServerManagerObserver;
use crate::server::tcpoutput::TcpOutputEvent::SendInitialInformation;
use commons::threading::AsyncJoin;
use commons::threading::channel::{ReceiveMetaData, SenderTrait};
use commons::threading::eventhandling::{ChannelEvent, ChannelEventResult, EventHandlerTrait, EventSenderTrait};
use commons::threading::eventhandling::WaitOrTryForNextEvent::{TryForNextEvent, WaitForNextEvent};
use crate::server::ServerCoreEvent::UdpPacket;
use crate::server::udphandler::UdpHandler;
use self::ServerCoreEvent::{StartListenerEvent, StartGameEvent, TcpConnectionEvent, GameTimerTick};

pub enum ServerCoreEvent<GameFactory: GameFactoryTrait> {
    //TODO: start listener before spawning event handler
    StartListenerEvent,

    //TODO: create render receiver sender before spawning event handler
    StartGameEvent(<GameFactory::Factory as FactoryTrait>::Sender<RenderReceiverMessage<GameFactory::Game>>),
    TcpConnectionEvent(TcpWriter<GameFactory>, TcpReader<GameFactory>),
    GameTimerTick,
    UdpPacket(SocketAddr, usize, [u8; MAX_UDP_DATAGRAM_SIZE])
}

pub struct ServerCore<GameFactory: GameFactoryTrait> {
    factory: GameFactory::Factory,
    sender: eventhandling::Sender<GameFactory::Factory, ServerCoreEvent<GameFactory>>,
    game_is_started: bool,
    server_config: ServerConfig,
    tcp_listener_sender_option: Option<eventhandling::Sender<GameFactory::Factory, ()>>,
    game_timer: Option<GameTimer<GameFactory::Factory, ServerGameTimerObserver<GameFactory>>>,
    tcp_inputs: Vec<eventhandling::Sender<GameFactory::Factory, ()>>,
    tcp_outputs: Vec<eventhandling::Sender<GameFactory::Factory, TcpOutputEvent<GameFactory::Game>>>,
    udp_socket: Option<UdpSocket>,
    udp_outputs: Vec<eventhandling::Sender<GameFactory::Factory, UdpOutputEvent<GameFactory::Game>>>,
    udp_input_sender_option: Option<eventhandling::Sender<GameFactory::Factory, ()>>,
    udp_handler: UdpHandler<GameFactory>,
    manager_sender_option: Option<eventhandling::Sender<GameFactory::Factory, ManagerEvent<GameFactory::Game>>>,
    render_receiver_sender: Option<<GameFactory::Factory as FactoryTrait>::Sender<RenderReceiverMessage<GameFactory::Game>>>,
    drop_steps_before: usize
}

impl<GameFactory: GameFactoryTrait> EventHandlerTrait for ServerCore<GameFactory> {
    type Event = ServerCoreEvent<GameFactory>;
    type ThreadReturn = ();

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> ChannelEventResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(_, StartListenerEvent) => self.start_listener(),
            ChannelEvent::ReceivedEvent(_, StartGameEvent(render_receiver_sender)) => self.start_game(render_receiver_sender),
            ChannelEvent::ReceivedEvent(_, TcpConnectionEvent(tcp_sender, tcp_receiver)) => self.on_tcp_connection(tcp_sender, tcp_receiver),
            ChannelEvent::ReceivedEvent(_, GameTimerTick) => self.on_game_timer_tick(),
            ChannelEvent::ReceivedEvent(_, UdpPacket(source, len, buf)) => self.on_udp_packet(source, len, buf),
            ChannelEvent::Timeout => Continue(WaitForNextEvent(self)),
            ChannelEvent::ChannelEmpty => Continue(WaitForNextEvent(self)),
            ChannelEvent::ChannelDisconnected => Break(()),
        }
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn { () }
}

impl<GameFactory: GameFactoryTrait> ServerCore<GameFactory> {

    pub fn new(factory: GameFactory::Factory, sender: eventhandling::Sender<GameFactory::Factory, ServerCoreEvent<GameFactory>>) -> Self {

        let server_config = ServerConfig::new(
            GameFactory::Game::STEP_PERIOD
        );

        let udp_handler = UdpHandler::new(factory.clone());

        Self {
            factory,
            sender,
            game_is_started: false,
            server_config,
            tcp_listener_sender_option: None,
            game_timer: None,
            tcp_inputs: Vec::new(),
            tcp_outputs: Vec::new(),
            udp_outputs: Vec::new(),
            drop_steps_before: 0,
            udp_socket: None,
            udp_input_sender_option: None,
            udp_handler,
            manager_sender_option: None,
            render_receiver_sender: None
        }
    }

    fn start_listener(mut self) -> ChannelEventResult<Self> {

        //TODO: on error, make sure other threads are closed
        //TODO: could these other threads be started somewhere else?

        let ip_addr_v4 = match Ipv4Addr::from_str("127.0.0.1") {
            Ok(ip_addr_v4) => ip_addr_v4,
            Err(error) => {
                error!("{:?}", error);
                return Break(());
            }
        };

        let socket_addr_v4 = SocketAddrV4::new(ip_addr_v4, GameFactory::Game::UDP_PORT);

        self.udp_socket = match UdpSocket::bind(socket_addr_v4) {
            Ok(udp_socket) => Some(udp_socket),
            Err(error) => {
                error!("{:?}", error);
                return Break(());
            }
        };

        let udp_input = match UdpInput::<GameFactory>::new(
            self.factory.clone(),
            self.udp_socket.as_ref().unwrap(),
            self.sender.clone()
        ) {
            Ok(udp_input) => udp_input,
            Err(error) => {
                error!("{:?}", error);
                return Break(());
            }
        };

        let udp_input_builder = self.factory.new_thread_builder()
            .name("ServerUdpInput")
            .spawn_listener(udp_input, AsyncJoin::log_async_join);

        self.udp_input_sender_option = Some(match udp_input_builder {
            Ok(udp_input_sender) => udp_input_sender,
            Err(error) => {
                error!("{:?}", error);
                return Break(());
            }
        });

        //Bind to TcpListener Socket
        let socket_addr_v4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), GameFactory::Game::TCP_PORT);
        let socket_addr = SocketAddr::from(socket_addr_v4);

        let tcp_listener_sender_result = self.factory.new_thread_builder()
            .name("ServerTcpListener")
            .spawn_tcp_listener(
                socket_addr,
                TcpConnectionHandler::<GameFactory>::new(self.sender.clone()),
                AsyncJoin::log_async_join
            );

        match tcp_listener_sender_result {
            Ok(tcp_listener_sender) => {
                self.tcp_listener_sender_option = Some(tcp_listener_sender);
                return Continue(TryForNextEvent(self));
            }
            Err(error) => {
                error!("Error starting Tcp Listener Thread: {:?}", error);
                return Break(());
            }
        }
    }

    pub(super) fn on_remote_udp_peer(&self, remote_udp_peer: RemoteUdpPeer) {
        if let Some(udp_output_sender) = self.udp_outputs.get(remote_udp_peer.get_player_index()) {
            udp_output_sender.send_event(UdpOutputEvent::RemotePeer(remote_udp_peer)).unwrap();
        }
    }

    fn start_game(mut self, render_receiver_sender: <GameFactory::Factory as FactoryTrait>::Sender<RenderReceiverMessage<GameFactory::Game>>) -> ChannelEventResult<Self> {
        //TODO: remove this line
        //let (render_receiver_sender, render_receiver) = RenderReceiver::<Game>::new();

        if !self.game_is_started {
            self.game_is_started = true;

            let initial_state = GameFactory::Game::get_initial_state(self.tcp_outputs.len());

            let mut udp_output_senders: Vec<eventhandling::Sender<GameFactory::Factory, UdpOutputEvent<GameFactory::Game>>> = Vec::new();

            for udp_output_sender in self.udp_outputs.iter() {
                udp_output_senders.push(udp_output_sender.clone());
            }

            let server_manager_observer = ServerManagerObserver::<GameFactory>::new(
                self.factory.clone(),
                udp_output_senders.clone(),
                render_receiver_sender.clone()
            );

            let manager_builder = self.factory.new_thread_builder()
                .name("ServerManager")
                .build_channel_for_event_handler::<Manager<ServerManagerObserver<GameFactory>>>();

            let server_game_timer_observer = ServerGameTimerObserver::new(
                self.factory.clone(),
                self.sender.clone()
            );

            let mut game_timer = GameTimer::new(
                self.factory.clone(),
                self.server_config,
                0,
                server_game_timer_observer
            );

            manager_builder.get_sender().send_event(ManagerEvent::DropStepsBeforeEvent(self.drop_steps_before)).unwrap();

            let server_initial_information = InitialInformation::<GameFactory::Game>::new(
                self.server_config.clone(),
                self.tcp_outputs.len(),
                usize::MAX,
                initial_state.clone()
            );

            manager_builder.get_sender().send_event(ManagerEvent::InitialInformationEvent(server_initial_information.clone())).unwrap();
            render_receiver_sender.send(RenderReceiverMessage::InitialInformation(server_initial_information.clone())).unwrap();

            for tcp_output in self.tcp_outputs.iter() {
                tcp_output.send_event(SendInitialInformation(
                    self.server_config.clone(),
                    self.tcp_outputs.len(),
                    initial_state.clone()
                )).unwrap();
            }

            game_timer.start_ticking();

            self.game_timer = Some(game_timer);

            self.manager_sender_option = Some(manager_builder.spawn_event_handler(Manager::new(self.factory.clone(), server_manager_observer), AsyncJoin::log_async_join).unwrap());
            self.render_receiver_sender = Some(render_receiver_sender);
        }

        return Continue(TryForNextEvent(self));
    }

    /*
    TODO:
    Server      Client
    Tcp Hello ->
        <- UdpHello
     */
    fn on_tcp_connection(mut self, tcp_sender: TcpWriter<GameFactory>, tcp_receiver: TcpReader<GameFactory>) -> ChannelEventResult<Self> {
        if !self.game_is_started {

            info!("TcpStream accepted: {:?}", tcp_sender.get_peer_addr());

            let player_index = self.tcp_inputs.len();

            let client_address = ClientAddress::new(player_index, tcp_sender.get_peer_addr().ip());

            let tcp_input_join_handle = self.factory.new_thread_builder()
                .name("ServerTcpInput")
                .spawn_tcp_reader(tcp_receiver, TcpInput::new(), AsyncJoin::log_async_join)
                .unwrap();

            self.tcp_inputs.push(tcp_input_join_handle);

            let udp_out_sender = self.factory.new_thread_builder()
                .name("ServerUdpOutput")
                .spawn_event_handler(
                    UdpOutput::<GameFactory>::new(self.factory.clone(), player_index, self.udp_socket.as_ref().unwrap()).unwrap(),
                    AsyncJoin::log_async_join)
                .unwrap();

            self.udp_handler.on_client_address(client_address);

            let tcp_output_sender = self.factory.new_thread_builder()
                .name("ServerTcpOutput")
                .spawn_event_handler(
                    TcpOutput::<GameFactory>::new(player_index, tcp_sender),
                    AsyncJoin::log_async_join
                )
                .unwrap();

            self.tcp_outputs.push(tcp_output_sender);
            self.udp_outputs.push(udp_out_sender);

        } else {
            info!("TcpStream connected after the core has stated and will be dropped. {:?}", tcp_sender.get_peer_addr());
        }

        return Continue(TryForNextEvent(self));
    }

    fn on_udp_packet(mut self, source: SocketAddr, len: usize, buf: [u8; MAX_UDP_DATAGRAM_SIZE]) -> ChannelEventResult<Self> {
        let (remote_peer, input_message) = self.udp_handler.on_udp_packet(len, buf, source);

        if let Some(remote_peer) = remote_peer {
            self.on_remote_udp_peer(remote_peer);
        }

        if let Some(input_message) = input_message {
            self.on_input_message(input_message);
        }

        return Continue(TryForNextEvent(self));
    }

    fn on_game_timer_tick(mut self) -> ChannelEventResult<Self> {

        let time_message = self.game_timer.as_ref().unwrap().create_timer_message();

        /*
        TODO: time is also sent directly fomr gametime observer, seems like this is a bug

        for udp_output in self.udp_outputs.iter() {
            udp_output.send_event(UdpOutputEvent::SendTimeMessage(time_message.clone()));
        }
        */

        self.drop_steps_before = time_message.get_step_from_actual_time(time_message.get_scheduled_time().subtract(GameFactory::Game::GRACE_PERIOD)).ceil() as usize;

        if let Some(manager_sender) = self.manager_sender_option.as_ref() {

            //the manager needs its lowest step to not have any new inputs
            if self.drop_steps_before > 1 {
                manager_sender.send_event(ManagerEvent::DropStepsBeforeEvent(self.drop_steps_before - 1)).unwrap();
            }
            manager_sender.send_event(ManagerEvent::SetRequestedStepEvent(time_message.get_step() + 1)).unwrap();
        }

        for udp_output in self.udp_outputs.iter() {
            udp_output.send_event(UdpOutputEvent::SendTimeMessage(time_message.clone())).unwrap();
        }

        self.render_receiver_sender.as_ref().unwrap().send(RenderReceiverMessage::TimeMessage(time_message.clone())).unwrap();

        return Continue(TryForNextEvent(self));
    }

    pub(super) fn on_input_message(&self, input_message: InputMessage<GameFactory::Game>) {

        //TODO: is game started?

        if self.drop_steps_before <= input_message.get_step() &&
            self.manager_sender_option.is_some() {

            self.manager_sender_option.as_ref()
                .unwrap()
                .send_event(ManagerEvent::InputEvent(input_message.clone()))
                .unwrap();

            for udp_output in self.udp_outputs.iter() {
                udp_output.send_event(UdpOutputEvent::SendInputMessage(input_message.clone())).unwrap();
            }
        }
    }
}
