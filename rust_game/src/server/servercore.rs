use std::net::{TcpStream, Ipv4Addr, SocketAddrV4, UdpSocket};

use log::info;
use crate::interface::GameTrait;
use crate::server::tcpinput::TcpInput;
use crate::threading::{ChannelDrivenThread, ChannelThread, Consumer, Sender};
use crate::server::{TcpListenerThread, ServerConfig};
use crate::server::tcpoutput::TcpOutput;
use crate::gametime::{GameTimer, TimeMessage};
use crate::gamemanager::{Manager, RenderReceiver};
use crate::messaging::{InputMessage, InitialInformation};
use std::str::FromStr;
use crate::server::udpinput::UdpInput;
use crate::server::udpoutput::UdpOutput;
use crate::server::clientaddress::ClientAddress;

//TODO: route game timer and player inputs through the core to
// get synchronous enforcement of the grace period

pub struct ServerCore<Game: GameTrait> {

    game_is_started: bool,
    server_config: ServerConfig,
    tcp_inputs: Vec<Sender<TcpInput>>,
    tcp_outputs: Vec<Sender<TcpOutput<Game>>>,
    udp_socket: Option<UdpSocket>,
    udp_outputs: Vec<Sender<UdpOutput<Game>>>,
    udp_input_sender: Option<Sender<UdpInput<Game>>>,
    manager_sender: Option<Sender<Manager<Game>>>,
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
            tcp_inputs: Vec::new(),
            tcp_outputs: Vec::new(),
            udp_outputs: Vec::new(),
            drop_steps_before: 0,
            udp_socket: None,
            udp_input_sender: None,
            manager_sender: None
        }
    }
}

impl<Game: GameTrait> Sender<ServerCore<Game>> {

    pub fn start_listener(&self) {
        let clone = self.clone();

        self.send(|core| {
            let ip_addr_v4 = Ipv4Addr::from_str("127.0.0.1").unwrap();
            let socket_addr_v4 = SocketAddrV4::new(ip_addr_v4, Game::UDP_PORT);
            core.udp_socket = Some(UdpSocket::bind(socket_addr_v4).unwrap());

            let udp_input = UdpInput::<Game>::new(core.udp_socket.as_ref().unwrap()).unwrap();
            let (udp_input_sender, udp_input_builder) = udp_input.build();
            udp_input_builder.name("ServerUdpInput").start().unwrap();
            core.udp_input_sender = Some(udp_input_sender);

            let (listener_sender, listener_builder) = TcpListenerThread::<Game>::new(clone).build();
            listener_builder.name("ServerTcpListener").start().unwrap();
        }).unwrap();
    }

    pub fn start_game(&self) -> RenderReceiver<Game> {
        let (render_receiver_sender, render_receiver) = RenderReceiver::<Game>::new();
        let core_sender = self.clone();
        self.send(move |core| {
            if !core.game_is_started {
                core.game_is_started = true;

                let initial_state = Game::get_initial_state(core.tcp_outputs.len());

                let (manager_sender, manager_builder) = Manager::<Game>::new(true, render_receiver_sender.clone()).build();
                let (timer_sender, timer_builder) = GameTimer::new(0).build();

                timer_sender.add_timer_message_consumer(core_sender.clone());
                timer_sender.add_timer_message_consumer(render_receiver_sender.clone());

                core.manager_sender = Some(manager_sender.clone());
                manager_sender.drop_steps_before(core.drop_steps_before);

                let server_initial_information = InitialInformation::<Game>::new(
                    core.server_config.clone(),
                    core.tcp_outputs.len(),
                    usize::MAX,
                    initial_state.clone());

                manager_sender.on_initial_information(server_initial_information.clone());
                render_receiver_sender.accept(server_initial_information.clone());
                timer_sender.accept(server_initial_information.clone());

                timer_sender.start().unwrap();

                for udp_output in core.udp_outputs.iter() {
                    timer_sender.add_timer_message_consumer(udp_output.clone());

                    manager_sender.add_completed_step_consumer(udp_output.clone());
                    manager_sender.add_server_input_consumer(udp_output.clone());
                }

                for tcp_output in core.tcp_outputs.iter() {
                    tcp_output.send_initial_information(
                        core.server_config.clone(),
                        core.tcp_outputs.len(),
                        initial_state.clone()
                    );
                }

                core.udp_input_sender.as_ref().unwrap().add_input_consumer(core_sender.clone()).unwrap();

                timer_builder.name("ServerTimer").start().unwrap();
                manager_builder.name("ServerManager").start().unwrap();
            }
        }).unwrap();

        return render_receiver;
    }

    pub fn on_tcp_connection(&self, tcp_stream: TcpStream) {
        self.send(move |core|{
            if !core.game_is_started {
                let player_index = core.tcp_inputs.len();

                let client_address = ClientAddress::new(player_index, tcp_stream.peer_addr().unwrap().ip());

                let (in_sender, in_thread_builder) = TcpInput::new(&tcp_stream).unwrap().build();
                in_thread_builder.name("ServerTcpInput").start().unwrap();
                core.tcp_inputs.push(in_sender);

                let (tcp_out_sender, tcp_out_builder) = TcpOutput::new(
                    player_index,
                    &tcp_stream
                ).unwrap().build();

                let (udp_out_sender, udp_out_builder) = UdpOutput::new(
                    player_index,
                    core.udp_socket.as_ref().unwrap()
                ).unwrap().build();

                let input_sender = core.udp_input_sender.as_ref().unwrap();
                input_sender.add_remote_peer_consumers(udp_out_sender.clone()).unwrap();
                input_sender.accept(client_address);

                tcp_out_builder.name("ServerTcpOutput").start().unwrap();
                udp_out_builder.name("ServerUdpOutput").start().unwrap();

                core.tcp_outputs.push(tcp_out_sender);
                core.udp_outputs.push(udp_out_sender);

                info!("TcpStream accepted: {:?}", tcp_stream);

            } else {
                info!("TcpStream connected after the core has stated and will be dropped. {:?}", tcp_stream);
            }
        }).unwrap();
    }
}

impl<Game: GameTrait> Consumer<TimeMessage> for Sender<ServerCore<Game>> {

    fn accept(&self, time_message: TimeMessage) {
        self.send(move |core|{
            core.drop_steps_before = time_message.get_step_from_actual_time(time_message.get_scheduled_time().subtract(Game::GRACE_PERIOD)).ceil() as usize;

            if core.manager_sender.is_some() {
                let manager_sender = core.manager_sender.as_ref().unwrap();

                //the manager needs its lowest step to not have any new inputs
                if core.drop_steps_before > 1 {
                    manager_sender.drop_steps_before(core.drop_steps_before - 1);
                }
                manager_sender.set_requested_step(time_message.get_step() + 1);
            }

        }).unwrap();
    }
}

impl<Game: GameTrait> Consumer<InputMessage<Game>> for Sender<ServerCore<Game>> {

    fn accept(&self, input_message: InputMessage<Game>) {
        self.send(move |core|{

            if core.drop_steps_before <= input_message.get_step() &&
                core.manager_sender.is_some() {

                core.manager_sender.as_ref().unwrap().accept(input_message.clone());
                for udp_output in core.udp_outputs.iter() {
                    udp_output.accept(input_message.clone());
                }
            }
        }).unwrap();
    }
}