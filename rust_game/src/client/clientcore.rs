use std::net::{Ipv4Addr, SocketAddrV4, SocketAddr, TcpStream, UdpSocket};
use std::str::FromStr;
use crate::gametime::{GameTimer, GameTimerEvent, TimeMessage};
use crate::client::tcpinput::TcpInput;
use crate::interface::GameTrait;
use crate::client::tcpoutput::TcpOutput;
use crate::messaging::{InitialInformation, InputMessage};
use crate::gamemanager::{Manager, RenderReceiver};
use log::{trace};
use crate::client::clientgametimeobserver::ClientGameTimerObserver;
use crate::client::clientmanagerobserver::ClientManagerObserver;
use crate::client::udpoutput::{UdpOutput, UdpOutputEvent};
use crate::client::udpinput::UdpInput;
use crate::threading::{ThreadBuilder, ChannelThread, ChannelDrivenThreadSender, ChannelDrivenThread, ThreadAction, OldThreadBuilderTrait, listener};
use crate::threading::eventhandling;
use crate::threading::eventhandling::EventSenderTrait;

pub struct ClientCore<Game: GameTrait> {
    server_ip: String,
    input_event_handler: Game::ClientInputEventHandler,
    manager_sender: Option<ChannelDrivenThreadSender<Manager<ClientManagerObserver<Game>>>>,
    timer_join_handle_option: Option<eventhandling::JoinHandle<GameTimerEvent<ClientGameTimerObserver<Game>>, ()>>,
    udp_input_join_handle_option: Option<eventhandling::JoinHandle<(), ()>>,
    udp_output_join_handle_option: Option<eventhandling::JoinHandle<UdpOutputEvent<Game>, ()>>,
    tcp_input_join_handle_option: Option<eventhandling::JoinHandle<(), ()>>,
    tcp_output_sender: Option<ChannelDrivenThreadSender<TcpOutput>>,
    initial_information: Option<InitialInformation<Game>>,
    last_time_message: Option<TimeMessage>
}

impl<Game: GameTrait> ClientCore<Game> {

    pub fn new(server_ip: &str) -> Self {

        ClientCore {server_ip: server_ip.to_string(),
            input_event_handler: Game::new_input_event_handler(),
            manager_sender: None,
            timer_join_handle_option: None,
            udp_input_join_handle_option: None,
            udp_output_join_handle_option: None,
            tcp_input_join_handle_option: None,
            tcp_output_sender: None,
            initial_information: None,
            last_time_message: None
        }
    }
}

impl<Game: GameTrait> ChannelDrivenThread<()> for ClientCore<Game> {

    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl<Game: GameTrait> ChannelDrivenThreadSender<ClientCore<Game>> {

    pub fn connect(&self) -> RenderReceiver<Game> {
        let (render_receiver_sender, render_receiver) = RenderReceiver::<Game>::new();
        let core_sender = self.clone();

        self.send(move |core|{
            let ip_addr_v4 = Ipv4Addr::from_str(core.server_ip.as_str()).unwrap();
            let socket_addr_v4 = SocketAddrV4::new(ip_addr_v4, Game::TCP_PORT);
            let socket_addr:SocketAddr = SocketAddr::from(socket_addr_v4);
            let tcp_stream = TcpStream::connect(socket_addr).unwrap();

            let server_udp_socket_addr_v4 = SocketAddrV4::new(ip_addr_v4, Game::UDP_PORT);

            let udp_socket = UdpSocket::bind("127.0.0.1:0").unwrap();

            let (manager_sender, manager_builder) =
                Manager::new(ClientManagerObserver::new(render_receiver_sender.clone())).build();

            let client_game_time_observer = ClientGameTimerObserver::new(
                core_sender.clone(),
                render_receiver_sender.clone());

            let game_timer_builder = ThreadBuilder::new()
                .name("ClientGameTimer")
                .build_channel_for_event_handler::<GameTimer<ClientGameTimerObserver<Game>>>();

            //TODO: find a better way to send this sender
            game_timer_builder.get_sender().send_event(GameTimerEvent::SetSender(game_timer_builder.clone_sender())).unwrap();

            let udp_output_builder = ThreadBuilder::new()
                .name("ClientUdpOutput")
                .build_channel_for_event_handler::<UdpOutput<Game>>();

            let (tcp_output_sender, tcp_output_builder) = TcpOutput::new(&tcp_stream).unwrap().build();

            let _manager_join_handle = manager_builder.name("ClientManager").start().unwrap();

            let tcp_input_join_handle = ThreadBuilder::new()
                .name("ClientTcpInput")
                .spawn_listener(TcpInput::new(
                    game_timer_builder.clone_sender(),
                    manager_sender.clone(),
                    core_sender.clone(),
                    udp_output_builder.clone_sender(),
                    render_receiver_sender.clone(),
                    &tcp_stream).unwrap())
                .unwrap();

            let _tcp_output_join_handle = tcp_output_builder.name("ClientTcpOutput").start().unwrap();

            let udp_output_join_handle = udp_output_builder.spawn_event_handler(
                UdpOutput::<Game>::new(server_udp_socket_addr_v4, &udp_socket).unwrap()
            ).unwrap();

            let udp_input_join_handle = ThreadBuilder::new()
                .name("ClientUdpInput")
                .spawn_listener(UdpInput::new(
                    server_udp_socket_addr_v4,
                    &udp_socket,
                    game_timer_builder.clone_sender(),
                    manager_sender.clone()
                ).unwrap())
                .unwrap();

            let game_timer_join_handle =  game_timer_builder.spawn_event_handler(GameTimer::new(
                Game::CLOCK_AVERAGE_SIZE,
                client_game_time_observer
            )).unwrap();

            core.timer_join_handle_option = Some(game_timer_join_handle);
            core.manager_sender = Some(manager_sender);
            core.tcp_output_sender = Some(tcp_output_sender);
            core.tcp_input_join_handle_option = Some(tcp_input_join_handle);
            core.udp_input_join_handle_option = Some(udp_input_join_handle);
            core.udp_output_join_handle_option = Some(udp_output_join_handle);

            return ThreadAction::Continue;
        }).unwrap();

        return render_receiver;
    }

    pub fn on_input_event(&self, input_event: Game::ClientInputEvent) {
        self.send(move |core|{
            if core.manager_sender.is_some() &&
                core.last_time_message.is_some() &&
                core.initial_information.is_some() {

                Game::handle_input_event(&mut core.input_event_handler, input_event);
            }

            return ThreadAction::Continue;
        }).unwrap();
    }

    pub fn on_initial_information(&self, initial_information: InitialInformation<Game>) {
        self.send(move |core|{
            core.initial_information = Some(initial_information);

            return ThreadAction::Continue;
        }).unwrap();
    }

    pub fn on_time_message(&self, time_message: TimeMessage) {
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
                let initial_information = core.initial_information.as_ref().unwrap();

                if time_message.get_step() > last_time_message.get_step() {
                    let message = InputMessage::<Game>::new(
                        //TODO: message or last message?
                        //TODO: define strict and consistent rules for how real time relates to ticks, input deadlines and display states
                        time_message.get_step(),
                        initial_information.get_player_index(),
                        Game::get_input(& mut core.input_event_handler)
                    );

                    manager_sender.on_input_message(message.clone());
                    core.udp_output_join_handle_option.as_ref().unwrap().get_sender().send_event(UdpOutputEvent::InputMessageEvent(message));

                    let client_drop_time = time_message.get_scheduled_time().subtract(Game::GRACE_PERIOD * 2);
                    let drop_step = time_message.get_step_from_actual_time(client_drop_time).ceil() as usize;

                    manager_sender.drop_steps_before(drop_step);
                    //TODO: message or last message or next?
                    //TODO: define strict and consistent rules for how real time relates to ticks, input deadlines and display states
                    manager_sender.set_requested_step(time_message.get_step() + 1);
                }
            }

            core.last_time_message = Some(time_message);

            return ThreadAction::Continue;
        }).unwrap();
    }
}

