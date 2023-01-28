use std::net::{TcpStream, Ipv4Addr, SocketAddrV4, UdpSocket};

use log::{error, info};
use crate::interface::GameTrait;
use crate::server::tcpinput::TcpInput;
use crate::threading::{ChannelDrivenThread, ChannelThread, ChannelDrivenThreadSender, ChannelDrivenThreadSenderError as SendError, ThreadAction, OldThreadBuilderTrait, listener, eventhandling, ThreadBuilder};
use crate::server::{TcpListenerThread, ServerConfig};
use crate::server::tcpoutput::TcpOutput;
use crate::gametime::{GameTimer, GameTimerEvent, TimeMessage};
use crate::gamemanager::{Manager, RenderReceiver, RenderReceiverMessage};
use crate::messaging::{InputMessage, InitialInformation};
use std::str::FromStr;
use crate::server::udpinput::{UdpInput, UdpInputEvent};
use crate::server::udpoutput::UdpOutput;
use crate::server::clientaddress::ClientAddress;
use crate::server::remoteudppeer::RemoteUdpPeer;
use crate::server::servergametimerobserver::ServerGameTimerObserver;
use crate::server::servermanagerobserver::ServerManagerObserver;

pub struct ServerCore<Game: GameTrait> {

    game_is_started: bool,
    server_config: ServerConfig,
    tcp_listener_join_handle_option: Option<eventhandling::JoinHandle<(), ()>>,
    timer_join_handle_option: Option<eventhandling::JoinHandle<GameTimerEvent<ServerGameTimerObserver<Game>>, ()>>,
    tcp_inputs: Vec<eventhandling::JoinHandle<(), ()>>,
    tcp_outputs: Vec<ChannelDrivenThreadSender<TcpOutput<Game>>>,
    udp_socket: Option<UdpSocket>,
    udp_outputs: Vec<ChannelDrivenThreadSender<UdpOutput<Game>>>,
    udp_input_join_handle_option: Option<eventhandling::JoinHandle<UdpInputEvent, ()>>,
    manager_sender: Option<ChannelDrivenThreadSender<Manager<ServerManagerObserver<Game>>>>,
    drop_steps_before: usize
}

impl<Game: GameTrait> ChannelDrivenThread<()> for ServerCore<Game> {
    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl<Game: GameTrait> ServerCore<Game> {

    pub fn new() -> Self {

        let server_config = ServerConfig::new(
            Game::STEP_PERIOD
        );

        Self {
            game_is_started: false,
            server_config,
            tcp_listener_join_handle_option: None,
            timer_join_handle_option: None,
            tcp_inputs: Vec::new(),
            tcp_outputs: Vec::new(),
            udp_outputs: Vec::new(),
            drop_steps_before: 0,
            udp_socket: None,
            udp_input_join_handle_option: None,
            manager_sender: None
        }
    }

}

impl<Game: GameTrait> ChannelDrivenThreadSender<ServerCore<Game>> {

    pub fn start_listener(&self) -> Result<(), SendError<ServerCore<Game>>> {

        let core_sender = self.clone();

        self.send(|core| {

            //TODO: on error, make sure other threads are closed
            //TODO: could these other threads be started somewhere else?

            let ip_addr_v4 = match Ipv4Addr::from_str("127.0.0.1") {
                Ok(ip_addr_v4) => ip_addr_v4,
                Err(error) => {
                    error!("{:?}", error);
                    return ThreadAction::Stop;
                }
            };

            let socket_addr_v4 = SocketAddrV4::new(ip_addr_v4, Game::UDP_PORT);

            core.udp_socket = match UdpSocket::bind(socket_addr_v4) {
                Ok(udp_socket) => Some(udp_socket),
                Err(error) => {
                    error!("{:?}", error);
                    return ThreadAction::Stop;
                }
            };

            let udp_input = match UdpInput::<Game>::new(
                core.udp_socket.as_ref().unwrap(),
                core_sender.clone()
            ) {
                Ok(udp_input) => udp_input,
                Err(error) => {
                    error!("{:?}", error);
                    return ThreadAction::Stop;
                }
            };

            let udp_input_builder = ThreadBuilder::new()
                .name("ServerUdpInput")
                .spawn_listener(udp_input);

            core.udp_input_join_handle_option = Some(match udp_input_builder {
                Ok(udp_input_join_handle) => udp_input_join_handle,
                Err(error) => {
                    error!("{:?}", error);
                    return ThreadAction::Stop;
                }
            });

            let tcp_listener_join_handle_result = ThreadBuilder::new()
                .name("ServerTcpListener")
                .spawn_listener(TcpListenerThread::<Game>::new(core_sender));

            match tcp_listener_join_handle_result {
                Ok(tcp_listener_join_handle) => {
                    core.tcp_listener_join_handle_option = Some(tcp_listener_join_handle);
                    return ThreadAction::Continue;
                }
                Err(error) => {
                    error!("Error starting Tcp Listener Thread: {:?}", error);
                    return ThreadAction::Stop;
                }
            }
        })
    }

    pub fn on_remote_udp_peer(&self, remote_udp_peer: RemoteUdpPeer) {
        self.send(|core|{

            if let Some(udp_output_sender) = core.udp_outputs.get(remote_udp_peer.get_player_index()) {
                udp_output_sender.on_remote_peer(remote_udp_peer);
            }

            return ThreadAction::Continue;
        }).unwrap();
    }

    pub fn start_game(&self) -> RenderReceiver<Game> {
        let (render_receiver_sender, render_receiver) = RenderReceiver::<Game>::new();
        let core_sender = self.clone();
        self.send(move |core| {
            if !core.game_is_started {
                core.game_is_started = true;

                let initial_state = Game::get_initial_state(core.tcp_outputs.len());

                let server_manager_observer = ServerManagerObserver::new(
                    core_sender.clone(),
                    core.udp_outputs.clone(),
                    render_receiver_sender.clone()
                );

                let (manager_sender, manager_builder) =
                    Manager::new(server_manager_observer).build();

                let server_game_timer_observer = ServerGameTimerObserver::new(
                    core_sender.clone(),
                    render_receiver_sender.clone(),
                    core.udp_outputs.clone()
                );

                let timer_builder = ThreadBuilder::new()
                    .name("ServerTimer")
                    .build_channel_for_event_handler::<GameTimer<ServerGameTimerObserver<Game>>>();

                core.manager_sender = Some(manager_sender.clone());
                manager_sender.drop_steps_before(core.drop_steps_before);

                let server_initial_information = InitialInformation::<Game>::new(
                    core.server_config.clone(),
                    core.tcp_outputs.len(),
                    usize::MAX,
                    initial_state.clone()
                );

                manager_sender.on_initial_information(server_initial_information.clone());
                render_receiver_sender.send(RenderReceiverMessage::InitialInformation(server_initial_information.clone())).unwrap();

                timer_builder.get_sender().send_event(GameTimerEvent::InitialInformationEvent(server_initial_information.clone())).unwrap();

                //TODO: fix this
                timer_builder.get_sender().send_event(GameTimerEvent::SetSender(timer_builder.clone_sender())).unwrap();

                timer_builder.get_sender().send_event(GameTimerEvent::StartTickingEvent).unwrap();

                for tcp_output in core.tcp_outputs.iter() {
                    tcp_output.send_initial_information(
                        core.server_config.clone(),
                        core.tcp_outputs.len(),
                        initial_state.clone()
                    );
                }

                core.timer_join_handle_option = Some(timer_builder.spawn_event_handler(GameTimer::new(
                    0,
                    server_game_timer_observer
                )).unwrap());

                manager_builder.name("ServerManager").start().unwrap();
            }

            return ThreadAction::Continue;
        }).unwrap();

        return render_receiver;
    }

    /*
    TODO:
    Server      Cliend
    Tcp Hello ->
        <- UdpHello
     */
    pub fn on_tcp_connection(&self, tcp_stream: TcpStream) -> Result<(), SendError<ServerCore<Game>>> {
        self.send(move |core|{
            if !core.game_is_started {
                let player_index = core.tcp_inputs.len();

                let client_address = ClientAddress::new(player_index, tcp_stream.peer_addr().unwrap().ip());

                let tcp_input_join_handle = ThreadBuilder::new()
                    .name("ServerTcpInput")
                    .spawn_listener(TcpInput::new(&tcp_stream).unwrap())
                    .unwrap();

                core.tcp_inputs.push(tcp_input_join_handle);

                let (tcp_out_sender, tcp_out_builder) = TcpOutput::new(
                    player_index,
                    &tcp_stream
                ).unwrap().build();

                let (udp_out_sender, udp_out_builder) = UdpOutput::new(
                    player_index,
                    core.udp_socket.as_ref().unwrap()
                ).unwrap().build();

                core.udp_input_join_handle_option.as_ref()
                    .unwrap()
                    .get_sender()
                    .send_event(UdpInputEvent::ClientAddress(client_address))
                    .unwrap();

                tcp_out_builder.name("ServerTcpOutput").start().unwrap();
                udp_out_builder.name("ServerUdpOutput").start().unwrap();

                core.tcp_outputs.push(tcp_out_sender);
                core.udp_outputs.push(udp_out_sender);

                info!("TcpStream accepted: {:?}", tcp_stream);

            } else {
                info!("TcpStream connected after the core has stated and will be dropped. {:?}", tcp_stream);
            }

            return ThreadAction::Continue;
        })
    }

    pub fn on_time_message(&self, time_message: TimeMessage) {
        self.send(move |core|{

            for udp_output in core.udp_outputs.iter() {
                udp_output.on_time_message(time_message.clone());
            }

            core.drop_steps_before = time_message.get_step_from_actual_time(time_message.get_scheduled_time().subtract(Game::GRACE_PERIOD)).ceil() as usize;

            if core.manager_sender.is_some() {
                let manager_sender = core.manager_sender.as_ref().unwrap();

                //the manager needs its lowest step to not have any new inputs
                if core.drop_steps_before > 1 {
                    manager_sender.drop_steps_before(core.drop_steps_before - 1);
                }
                manager_sender.set_requested_step(time_message.get_step() + 1);
            }

            return ThreadAction::Continue;
        }).unwrap();
    }

    pub fn on_input_message(&self, input_message: InputMessage<Game>) {
        self.send(move |core|{

            //TODO: is game started?

            if core.drop_steps_before <= input_message.get_step() &&
                core.manager_sender.is_some() {

                core.manager_sender.as_ref().unwrap().on_input_message(input_message.clone());
                for udp_output in core.udp_outputs.iter() {
                    udp_output.on_input_message(input_message.clone());
                }
            }

            return ThreadAction::Continue;
        }).unwrap();
    }
}
