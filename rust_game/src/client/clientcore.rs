use std::net::{Ipv4Addr, SocketAddrV4, SocketAddr, TcpStream, UdpSocket};
use std::ops::ControlFlow::{Break, Continue};
use std::str::FromStr;
use crate::gametime::{GameTimer, GameTimerEvent, TimeMessage};
use crate::client::tcpinput::TcpInput;
use crate::interface::GameTrait;
use crate::client::tcpoutput::TcpOutput;
use crate::messaging::{InitialInformation, InputMessage};
use crate::gamemanager::{Manager, ManagerEvent, RenderReceiverMessage};
use log::{trace};
use crate::client::clientcore::ClientCoreEvent::{Connect, OnInitialInformation, OnInputEvent, OnTimeMessage};
use crate::client::clientgametimeobserver::ClientGameTimerObserver;
use crate::client::clientmanagerobserver::ClientManagerObserver;
use crate::client::udpoutput::{UdpOutput, UdpOutputEvent};
use crate::client::udpinput::UdpInput;
use crate::threading::{ThreadBuilder, ChannelThread, ChannelDrivenThreadSender, OldThreadBuilderTrait, listener};
use crate::threading::channel::{ReceiveMetaData, Sender};
use crate::threading::eventhandling;
use crate::threading::eventhandling::{ChannelEvent, ChannelEventResult, EventHandlerTrait};
use crate::threading::eventhandling::WaitOrTryForNextEvent::{TryForNextEvent, WaitForNextEvent};

pub enum ClientCoreEvent<Game: GameTrait> {
    Connect(Sender<RenderReceiverMessage<Game>>),
    OnInitialInformation(InitialInformation<Game>),
    OnInputEvent(Game::ClientInputEvent),
    OnTimeMessage(TimeMessage)
}

//TODO: make this an enum, get rid of all the options
pub struct ClientCore<Game: GameTrait> {
    sender: eventhandling::Sender<ClientCoreEvent<Game>>,
    server_ip: String,
    input_event_handler: Game::ClientInputEventHandler,
    manager_join_handle_option: Option<eventhandling::JoinHandle<Manager<ClientManagerObserver<Game>>>>,
    timer_join_handle_option: Option<eventhandling::JoinHandle<GameTimer<ClientGameTimerObserver<Game>>>>,
    udp_input_join_handle_option: Option<listener::JoinHandle<UdpInput<Game>>>,
    udp_output_join_handle_option: Option<eventhandling::JoinHandle<UdpOutput<Game>>>,
    tcp_input_join_handle_option: Option<listener::JoinHandle<TcpInput<Game>>>,
    tcp_output_join_handle_option: Option<eventhandling::JoinHandle<TcpOutput>>,
    initial_information: Option<InitialInformation<Game>>,
    last_time_message: Option<TimeMessage>
}

impl<Game: GameTrait> ClientCore<Game> {

    pub fn new(server_ip: &str, sender: eventhandling::Sender<ClientCoreEvent<Game>>) -> Self {

        ClientCore {
            sender,
            server_ip: server_ip.to_string(),
            input_event_handler: Game::new_input_event_handler(),
            manager_join_handle_option: None,
            timer_join_handle_option: None,
            udp_input_join_handle_option: None,
            udp_output_join_handle_option: None,
            tcp_input_join_handle_option: None,
            tcp_output_join_handle_option: None,
            initial_information: None,
            last_time_message: None
        }
    }

    fn connect(mut self, render_receiver_sender: Sender<RenderReceiverMessage<Game>>) -> ChannelEventResult<Self> {

        let ip_addr_v4 = Ipv4Addr::from_str(self.server_ip.as_str()).unwrap();
        let socket_addr_v4 = SocketAddrV4::new(ip_addr_v4, Game::TCP_PORT);
        let socket_addr:SocketAddr = SocketAddr::from(socket_addr_v4);
        let tcp_stream = TcpStream::connect(socket_addr).unwrap();

        let server_udp_socket_addr_v4 = SocketAddrV4::new(ip_addr_v4, Game::UDP_PORT);

        let udp_socket = UdpSocket::bind("127.0.0.1:0").unwrap();

        let client_game_time_observer = ClientGameTimerObserver::new(
            self.sender.clone(),
            render_receiver_sender.clone());

        let game_timer_builder = ThreadBuilder::new()
            .name("ClientGameTimer")
            .build_channel_for_event_handler::<GameTimer<ClientGameTimerObserver<Game>>>();

        //TODO: find a better way to send this sender
        game_timer_builder.get_sender().send_event(GameTimerEvent::SetSender(game_timer_builder.clone_sender())).unwrap();

        let udp_output_builder = ThreadBuilder::new()
            .name("ClientUdpOutput")
            .build_channel_for_event_handler::<UdpOutput<Game>>();

        let manager_join_handle = ThreadBuilder::new()
            .name("ClientManager")
            .spawn_event_handler(Manager::new(ClientManagerObserver::new(render_receiver_sender.clone())))
            .unwrap();

        let tcp_input_join_handle = ThreadBuilder::new()
            .name("ClientTcpInput")
            .spawn_listener(TcpInput::new(
                game_timer_builder.clone_sender(),
                manager_join_handle.get_sender().clone(),
                self.sender.clone(),
                udp_output_builder.clone_sender(),
                render_receiver_sender.clone(),
                &tcp_stream).unwrap())
            .unwrap();

        let tcp_output_join_handle = ThreadBuilder::new()
            .name("ClientTcpOutput")
            .spawn_event_handler(TcpOutput::new(&tcp_stream).unwrap())
            .unwrap();

        let udp_output_join_handle = udp_output_builder.spawn_event_handler(
            UdpOutput::<Game>::new(server_udp_socket_addr_v4, &udp_socket).unwrap()
        ).unwrap();

        let udp_input_join_handle = ThreadBuilder::new()
            .name("ClientUdpInput")
            .spawn_listener(UdpInput::new(
                server_udp_socket_addr_v4,
                &udp_socket,
                game_timer_builder.clone_sender(),
                manager_join_handle.get_sender().clone()
            ).unwrap())
            .unwrap();

        let game_timer_join_handle =  game_timer_builder.spawn_event_handler(GameTimer::new(
            Game::CLOCK_AVERAGE_SIZE,
            client_game_time_observer
        )).unwrap();

        self.timer_join_handle_option = Some(game_timer_join_handle);
        self.manager_join_handle_option = Some(manager_join_handle);
        self.tcp_output_join_handle_option = Some(tcp_output_join_handle);
        self.tcp_input_join_handle_option = Some(tcp_input_join_handle);
        self.udp_input_join_handle_option = Some(udp_input_join_handle);
        self.udp_output_join_handle_option = Some(udp_output_join_handle);

        return Continue(TryForNextEvent(self));
    }

    fn on_input_event(mut self, input_event: Game::ClientInputEvent) -> ChannelEventResult<Self> {

        if self.manager_join_handle_option.is_some() &&
            self.last_time_message.is_some() &&
            self.initial_information.is_some() {

            Game::handle_input_event(&mut self.input_event_handler, input_event);
        }

        return Continue(TryForNextEvent(self));
    }

    fn on_initial_information(mut self, initial_information: InitialInformation<Game>) -> ChannelEventResult<Self> {
        self.initial_information = Some(initial_information);

        return Continue(TryForNextEvent(self));
    }

    fn on_time_message(mut self, time_message: TimeMessage) -> ChannelEventResult<Self> {

        trace!("TimeMessage step_index: {:?}", time_message.get_step());

        //TODO: check if this tick is really the next tick?
        //TODO: log a warn if a tick is missed or out of order
        if self.last_time_message.is_some() &&
            self.tcp_output_join_handle_option.is_some() &&
            self.initial_information.is_some() &&
            self.manager_join_handle_option.is_some() {

            let manager_sender = self.manager_join_handle_option.as_ref().unwrap().get_sender();
            let last_time_message = self.last_time_message.as_ref().unwrap();
            let initial_information = self.initial_information.as_ref().unwrap();

            if time_message.get_step() > last_time_message.get_step() {
                let message = InputMessage::<Game>::new(
                    //TODO: message or last message?
                    //TODO: define strict and consistent rules for how real time relates to ticks, input deadlines and display states
                    time_message.get_step(),
                    initial_information.get_player_index(),
                    Game::get_input(& mut self.input_event_handler)
                );

                manager_sender.send_event(ManagerEvent::InputEvent(message.clone())).unwrap();
                self.udp_output_join_handle_option.as_ref().unwrap().get_sender().send_event(UdpOutputEvent::InputMessageEvent(message));

                let client_drop_time = time_message.get_scheduled_time().subtract(Game::GRACE_PERIOD * 2);
                let drop_step = time_message.get_step_from_actual_time(client_drop_time).ceil() as usize;

                manager_sender.send_event(ManagerEvent::DropStepsBeforeEvent(drop_step)).unwrap();
                //TODO: message or last message or next?
                //TODO: define strict and consistent rules for how real time relates to ticks, input deadlines and display states
                manager_sender.send_event(ManagerEvent::SetRequestedStepEvent(time_message.get_step() + 1)).unwrap();
            }
        }

        self.last_time_message = Some(time_message);

        return Continue(TryForNextEvent(self));
    }
}

impl<Game: GameTrait> EventHandlerTrait for ClientCore<Game> {
    type Event = ClientCoreEvent<Game>;
    type ThreadReturn = ();

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> ChannelEventResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(_, Connect(render_receiver_sender)) => self.connect(render_receiver_sender),
            ChannelEvent::ReceivedEvent(_, OnInitialInformation(initial_information)) => self.on_initial_information(initial_information),
            ChannelEvent::ReceivedEvent(_, OnInputEvent(client_input_event)) => self.on_input_event(client_input_event),
            ChannelEvent::ReceivedEvent(_, OnTimeMessage(time_message)) => self.on_time_message(time_message),
            ChannelEvent::ChannelEmpty => Continue(WaitForNextEvent(self)),
            ChannelEvent::ChannelDisconnected => Break(())
        }
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn { () }
}