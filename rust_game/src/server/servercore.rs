use std::net::{TcpStream, Ipv4Addr, SocketAddrV4, UdpSocket};
use std::ops::ControlFlow::{Continue, Break};

use log::{error, info};
use crate::interface::GameTrait;
use crate::server::tcpinput::TcpInput;
use commons::threading::{eventhandling, ThreadBuilder};
use crate::server::{TcpListenerThread, ServerConfig};
use crate::server::tcpoutput::{TcpOutput, TcpOutputEvent};
use crate::gametime::{GameTimer, GameTimerEvent, TimeMessage};
use crate::gamemanager::{Manager, ManagerEvent, RenderReceiverMessage};
use crate::messaging::{InputMessage, InitialInformation};
use std::str::FromStr;
use crate::server::udpinput::{UdpInput, UdpInputEvent};
use crate::server::udpoutput::{UdpOutput, UdpOutputEvent};
use crate::server::clientaddress::ClientAddress;
use crate::server::remoteudppeer::RemoteUdpPeer;
use crate::server::servergametimerobserver::ServerGameTimerObserver;
use crate::server::servermanagerobserver::ServerManagerObserver;
use crate::server::tcpoutput::TcpOutputEvent::SendInitialInformation;
use commons::threading::AsyncJoin;
use commons::threading::channel;
use commons::threading::channel::ReceiveMetaData;
use commons::threading::eventhandling::{ChannelEvent, ChannelEventResult, EventHandlerTrait};
use commons::threading::eventhandling::WaitOrTryForNextEvent::{TryForNextEvent, WaitForNextEvent};
use self::ServerCoreEvent::{StartListenerEvent, RemoteUdpPeerEvent, StartGameEvent, TcpConnectionEvent, TimeMessageEvent, InputMessageEvent};

pub enum ServerCoreEvent<Game: GameTrait> {
    //TODO: start listener before spawning event handler
    StartListenerEvent,

    RemoteUdpPeerEvent(RemoteUdpPeer),

    //TODO: create render receiver sender before spawning event handler
    StartGameEvent(channel::Sender<RenderReceiverMessage<Game>>),
    TcpConnectionEvent(TcpStream),
    TimeMessageEvent(TimeMessage),
    InputMessageEvent(InputMessage<Game>)
}

pub struct ServerCore<Game: GameTrait> {
    sender: eventhandling::Sender<ServerCoreEvent<Game>>,
    game_is_started: bool,
    server_config: ServerConfig,
    tcp_listener_sender_option: Option<eventhandling::Sender<()>>,
    timer_sender_option: Option<eventhandling::Sender<GameTimerEvent<ServerGameTimerObserver<Game>>>>,
    tcp_inputs: Vec<eventhandling::Sender<()>>,
    tcp_outputs: Vec<eventhandling::Sender<TcpOutputEvent<Game>>>,
    udp_socket: Option<UdpSocket>,
    udp_outputs: Vec<eventhandling::Sender<UdpOutputEvent<Game>>>,
    udp_input_sender_option: Option<eventhandling::Sender<UdpInputEvent>>,
    manager_sender_option: Option<eventhandling::Sender<ManagerEvent<Game>>>,
    drop_steps_before: usize
}

impl<Game: GameTrait> EventHandlerTrait for ServerCore<Game> {
    type Event = ServerCoreEvent<Game>;
    type ThreadReturn = ();

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> ChannelEventResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(_, StartListenerEvent) => self.start_listener(),
            ChannelEvent::ReceivedEvent(_, RemoteUdpPeerEvent(remote_udp_peer)) => self.on_remote_udp_peer(remote_udp_peer),
            ChannelEvent::ReceivedEvent(_, StartGameEvent(render_receiver_sender)) => self.start_game(render_receiver_sender),
            ChannelEvent::ReceivedEvent(_, TcpConnectionEvent(tcp_stream)) => self.on_tcp_connection(tcp_stream),
            ChannelEvent::ReceivedEvent(_, TimeMessageEvent(time_message)) => self.on_time_message(time_message),
            ChannelEvent::ReceivedEvent(_, InputMessageEvent(input_message)) => self.on_input_message(input_message),
            ChannelEvent::ChannelEmpty => Continue(WaitForNextEvent(self)),
            ChannelEvent::ChannelDisconnected => Break(()),
        }
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn { () }
}

impl<Game: GameTrait> ServerCore<Game> {

    pub fn new(sender: eventhandling::Sender<ServerCoreEvent<Game>>) -> Self {

        let server_config = ServerConfig::new(
            Game::STEP_PERIOD
        );

        Self {
            sender,
            game_is_started: false,
            server_config,
            tcp_listener_sender_option: None,
            timer_sender_option: None,
            tcp_inputs: Vec::new(),
            tcp_outputs: Vec::new(),
            udp_outputs: Vec::new(),
            drop_steps_before: 0,
            udp_socket: None,
            udp_input_sender_option: None,
            manager_sender_option: None
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

        let socket_addr_v4 = SocketAddrV4::new(ip_addr_v4, Game::UDP_PORT);

        self.udp_socket = match UdpSocket::bind(socket_addr_v4) {
            Ok(udp_socket) => Some(udp_socket),
            Err(error) => {
                error!("{:?}", error);
                return Break(());
            }
        };

        let udp_input = match UdpInput::<Game>::new(
            self.udp_socket.as_ref().unwrap(),
            self.sender.clone()
        ) {
            Ok(udp_input) => udp_input,
            Err(error) => {
                error!("{:?}", error);
                return Break(());
            }
        };

        let udp_input_builder = ThreadBuilder::new()
            .name("ServerUdpInput")
            .spawn_listener(udp_input, AsyncJoin::log_async_join);

        self.udp_input_sender_option = Some(match udp_input_builder {
            Ok(udp_input_sender) => udp_input_sender,
            Err(error) => {
                error!("{:?}", error);
                return Break(());
            }
        });

        let tcp_listener_sender_result = ThreadBuilder::new()
            .name("ServerTcpListener")
            .spawn_listener(TcpListenerThread::<Game>::new(self.sender.clone()), AsyncJoin::log_async_join);

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

    fn on_remote_udp_peer(self, remote_udp_peer: RemoteUdpPeer) -> ChannelEventResult<Self> {
        if let Some(udp_output_sender) = self.udp_outputs.get(remote_udp_peer.get_player_index()) {
            udp_output_sender.send_event(UdpOutputEvent::RemotePeer(remote_udp_peer)).unwrap();
        }

        return Continue(TryForNextEvent(self));
    }

    fn start_game(mut self, render_receiver_sender: channel::Sender<RenderReceiverMessage<Game>>) -> ChannelEventResult<Self> {
        //TODO: remove this line
        //let (render_receiver_sender, render_receiver) = RenderReceiver::<Game>::new();

        if !self.game_is_started {
            self.game_is_started = true;

            let initial_state = Game::get_initial_state(self.tcp_outputs.len());

            let mut udp_output_senders: Vec<eventhandling::Sender<UdpOutputEvent<Game>>> = Vec::new();

            for udp_output_sender in self.udp_outputs.iter() {
                udp_output_senders.push(udp_output_sender.clone());
            }

            let server_manager_observer = ServerManagerObserver::new(
                udp_output_senders.clone(),
                render_receiver_sender.clone()
            );

            let manager_builder = ThreadBuilder::new()
                .name("ServerManager")
                .build_channel_for_event_handler::<Manager<ServerManagerObserver<Game>>>();

            let server_game_timer_observer = ServerGameTimerObserver::new(
                self.sender.clone(),
                render_receiver_sender.clone(),
                udp_output_senders
            );

            let timer_builder = ThreadBuilder::new()
                .name("ServerTimer")
                .build_channel_for_event_handler::<GameTimer<ServerGameTimerObserver<Game>>>();

            manager_builder.get_sender().send_event(ManagerEvent::DropStepsBeforeEvent(self.drop_steps_before)).unwrap();

            let server_initial_information = InitialInformation::<Game>::new(
                self.server_config.clone(),
                self.tcp_outputs.len(),
                usize::MAX,
                initial_state.clone()
            );

            manager_builder.get_sender().send_event(ManagerEvent::InitialInformationEvent(server_initial_information.clone())).unwrap();
            render_receiver_sender.send(RenderReceiverMessage::InitialInformation(server_initial_information.clone())).unwrap();

            timer_builder.get_sender().send_event(GameTimerEvent::InitialInformationEvent(server_initial_information.clone())).unwrap();

            timer_builder.get_sender().send_event(GameTimerEvent::StartTickingEvent).unwrap();

            for tcp_output in self.tcp_outputs.iter() {
                tcp_output.send_event(SendInitialInformation(
                    self.server_config.clone(),
                    self.tcp_outputs.len(),
                    initial_state.clone()
                )).unwrap();
            }

            let game_timer = GameTimer::new(
                0,
                server_game_timer_observer,
                timer_builder.clone_sender()
            );

            self.timer_sender_option = Some(timer_builder.spawn_event_handler(game_timer, AsyncJoin::log_async_join).unwrap());

            self.manager_sender_option = Some(manager_builder.spawn_event_handler(Manager::new(server_manager_observer), AsyncJoin::log_async_join).unwrap());

        }

        return Continue(TryForNextEvent(self));
    }

    /*
    TODO:
    Server      Cliend
    Tcp Hello ->
        <- UdpHello
     */
    fn on_tcp_connection(mut self, tcp_stream: TcpStream) -> ChannelEventResult<Self> {
        if !self.game_is_started {
            let player_index = self.tcp_inputs.len();

            let client_address = ClientAddress::new(player_index, tcp_stream.peer_addr().unwrap().ip());

            let tcp_input_join_handle = ThreadBuilder::new()
                .name("ServerTcpInput")
                .spawn_listener(TcpInput::new(&tcp_stream).unwrap(), AsyncJoin::log_async_join)
                .unwrap();

            self.tcp_inputs.push(tcp_input_join_handle);

            let udp_out_sender = ThreadBuilder::new()
                .name("ServerUdpOutput")
                .spawn_event_handler(
                    UdpOutput::new(player_index, self.udp_socket.as_ref().unwrap()).unwrap(),
                    AsyncJoin::log_async_join)
                .unwrap();

            self.udp_input_sender_option.as_ref()
                .unwrap()
                .send_event(UdpInputEvent::ClientAddress(client_address))
                .unwrap();

            let tcp_output_sender = ThreadBuilder::new()
                .name("ServerTcpOutput")
                .spawn_event_handler(
                    TcpOutput::new(player_index, &tcp_stream).unwrap(),
                    AsyncJoin::log_async_join
                )
                .unwrap();

            self.tcp_outputs.push(tcp_output_sender);
            self.udp_outputs.push(udp_out_sender);

            info!("TcpStream accepted: {:?}", tcp_stream);

        } else {
            info!("TcpStream connected after the core has stated and will be dropped. {:?}", tcp_stream);
        }

        return Continue(TryForNextEvent(self));
    }

    fn on_time_message(mut self, time_message: TimeMessage) -> ChannelEventResult<Self> {
            /*
            TODO: time is also sent directly fomr gametime observer, seems like this is a bug

            for udp_output in self.udp_outputs.iter() {
                udp_output.send_event(UdpOutputEvent::SendTimeMessage(time_message.clone()));
            }
            */

            self.drop_steps_before = time_message.get_step_from_actual_time(time_message.get_scheduled_time().subtract(Game::GRACE_PERIOD)).ceil() as usize;

            if let Some(manager_sender) = self.manager_sender_option.as_ref() {

                //the manager needs its lowest step to not have any new inputs
                if self.drop_steps_before > 1 {
                    manager_sender.send_event(ManagerEvent::DropStepsBeforeEvent(self.drop_steps_before - 1)).unwrap();
                }
                manager_sender.send_event(ManagerEvent::SetRequestedStepEvent(time_message.get_step() + 1)).unwrap();
            }

            return Continue(TryForNextEvent(self));
    }

    fn on_input_message(self, input_message: InputMessage<Game>) -> ChannelEventResult<Self> {

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

        return Continue(TryForNextEvent(self));
    }
}