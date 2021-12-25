use std::net::{Ipv4Addr, SocketAddrV4, SocketAddr, TcpStream, UdpSocket};
use std::str::FromStr;
use crate::gametime::{TimeDuration, GameTimer, TimeMessage};
use crate::threading::{ChannelThread, Sender, ChannelDrivenThread, Consumer};
use crate::client::tcpinput::TcpInput;
use crate::interface::Game;
use crate::client::tcpoutput::TcpOutput;
use crate::messaging::{InitialInformation, InputMessage};
use crate::gamemanager::{Manager, RenderReceiver};
use log::{info, trace};
use crate::client::udpoutput::UdpOutput;
use crate::client::udpinput::UdpInput;

pub struct Core<GameType: Game> {
    server_ip: String,
    tcp_port: u16,
    udp_port: u16,
    //TODO: remove
    step_duration: TimeDuration,
    grace_period: TimeDuration,
    clock_average_size: usize,
    input_event_handler: GameType::InputEventHandlerType,
    manager_sender: Option<Sender<Manager<GameType>>>,
    udp_output_sender: Option<Sender<UdpOutput<GameType>>>,
    tcp_output_sender: Option<Sender<TcpOutput>>,
    initial_information: Option<InitialInformation<GameType>>,
    last_time_message: Option<TimeMessage>
}

impl<GameType: Game> Core<GameType> {

    pub fn new(server_ip: &str,
               tcp_port: u16,
               udp_port: u16,
               step_duration: TimeDuration,
               grace_period: TimeDuration,
               clock_average_size: usize) -> Self {

        Core{server_ip: server_ip.to_string(),
            tcp_port,
            udp_port,
            step_duration,
            grace_period,
            clock_average_size,
            input_event_handler: GameType::new_input_event_handler(),
            manager_sender: None,
            udp_output_sender: None,
            tcp_output_sender: None,
            initial_information: None,
            last_time_message: None
        }
    }
}

impl<GameType: Game> ChannelDrivenThread<()> for Core<GameType> {

    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl<GameType: Game> Sender<Core<GameType>> {

    pub fn connect(&self) -> RenderReceiver<GameType> {
        let (render_receiver_sender, render_receiver) = RenderReceiver::<GameType>::new();
        let core_sender = self.clone();

        self.send(move |core|{
            let ip_addr_v4 = Ipv4Addr::from_str(core.server_ip.as_str()).unwrap();
            let socket_addr_v4 = SocketAddrV4::new(ip_addr_v4, core.tcp_port);
            let socket_addr:SocketAddr = SocketAddr::from(socket_addr_v4);
            let tcp_stream = TcpStream::connect(socket_addr).unwrap();

            let server_udp_socket_addr_v4 = SocketAddrV4::new(ip_addr_v4, core.udp_port);

            let udp_socket = UdpSocket::bind("127.0.0.1:0").unwrap();

            let (manager_sender, manager_builder) = Manager::<GameType>::new(false, core.grace_period).build();
            let (game_timer_sender, game_timer_builder) = GameTimer::new(core.clock_average_size).build();
            let (tcp_input_sender, tcp_input_builder) = TcpInput::<GameType>::new(&tcp_stream).unwrap().build();
            let (tcp_output_sender, tcp_output_builder) = TcpOutput::new(&tcp_stream).unwrap().build();
            let (udp_output_sender, udp_output_builder) = UdpOutput::<GameType>::new(server_udp_socket_addr_v4, &udp_socket).unwrap().build();
            let (udp_input_sender, udp_input_builder) = UdpInput::<GameType>::new(server_udp_socket_addr_v4, &udp_socket).unwrap().build();

            tcp_input_sender.add_initial_information_message_consumer(manager_sender.clone());
            tcp_input_sender.add_initial_information_message_consumer(core_sender.clone());
            tcp_input_sender.add_initial_information_message_consumer(udp_output_sender.clone());
            tcp_input_sender.add_initial_information_message_consumer(render_receiver_sender.clone());
            tcp_input_sender.add_initial_information_message_consumer(game_timer_sender.clone());

            udp_input_sender.add_time_message_consumer(game_timer_sender.clone()).unwrap();
            udp_input_sender.add_input_message_consumer(manager_sender.clone());
            udp_input_sender.add_server_input_message_consumer(manager_sender.clone());
            udp_input_sender.add_state_message_consumer(manager_sender.clone());

            game_timer_sender.add_timer_message_consumer(core_sender.clone());
            game_timer_sender.add_timer_message_consumer(render_receiver_sender.clone());

            manager_sender.add_requested_step_consumer(render_receiver_sender.clone());

            let _manager_join_handle = manager_builder.name("ClientManager").start().unwrap();
            let _tcp_input_join_handle = tcp_input_builder.name("ClientTcpInput").start().unwrap();
            let _tcp_output_join_handle = tcp_output_builder.name("ClientTcpOutput").start().unwrap();
            let _udp_output_join_handle = udp_output_builder.name("ClientUdpOutput").start().unwrap();
            let _udp_input_join_handle = udp_input_builder.name("ClientUdpInput").start().unwrap();
            let _game_timer_join_handle = game_timer_builder.name("ClientGameTimer").start().unwrap();

            core.manager_sender = Some(manager_sender);
            core.tcp_output_sender = Some(tcp_output_sender);
            core.udp_output_sender = Some(udp_output_sender);

        }).unwrap();

        return render_receiver;
    }

    pub fn onInputEvent(&self, input_event: GameType::InputEventType) {
        self.send(move |core|{
            if core.manager_sender.is_some() &&
                core.last_time_message.is_some() &&
                core.initial_information.is_some() {

                GameType::handle_input_event(&mut core.input_event_handler, input_event);
            }
        }).unwrap();
    }
}

impl<GameType: Game> Consumer<InitialInformation<GameType>> for Sender<Core<GameType>> {

    fn accept(&self, initial_information: InitialInformation<GameType>) {
        self.send(move |core|{
            info!("InitialInformation Received.");
            core.initial_information = Some(initial_information);
        }).unwrap();
    }
}

impl<GameType: Game> Consumer<TimeMessage> for Sender<Core<GameType>> {

    fn accept(&self, time_message: TimeMessage) {

        self.send(move |core|{

            trace!("TimeMessage step_index: {:?}", time_message.get_step());

            //TODO: check if this tick is really the next tick?
            //TODO: log a warn if a tick is missed or out of order
            if core.last_time_message.is_some() &&
                core.tcp_output_sender.is_some() &&
                core.initial_information.is_some() &&
                core.manager_sender.is_some() {

                let manager_sender = core.manager_sender.as_ref().unwrap();
                let last_time_message = core.last_time_message.as_ref().unwrap();
                let udp_output_sender = core.udp_output_sender.as_ref().unwrap();
                let initial_information = core.initial_information.as_ref().unwrap();

                if time_message.get_step() > last_time_message.get_step() {
                    let message = InputMessage::<GameType>::new(
                        //TODO: message or last message?
                        //TODO: define strict and consistent rules for how real time relates to ticks, input deadlines and display states
                        time_message.get_step(),
                        initial_information.get_player_index(),
                        GameType::get_input(& mut core.input_event_handler)
                    );

                    manager_sender.accept(message.clone());
                    udp_output_sender.accept(message);

                    let client_drop_time = time_message.get_scheduled_time().subtract(core.grace_period * 2);
                    let drop_step = time_message.get_step_from_actual_time(client_drop_time).ceil() as usize;

                    manager_sender.drop_steps_before(drop_step);
                    //TODO: message or last message or next?
                    //TODO: define strict and consistent rules for how real time relates to ticks, input deadlines and display states
                    manager_sender.set_requested_step(time_message.get_step() + 1);
                }
            }

            core.last_time_message = Some(time_message);

        }).unwrap();
    }
}

