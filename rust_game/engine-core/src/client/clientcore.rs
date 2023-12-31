use std::net::{Ipv4Addr, SocketAddrV4, SocketAddr};
use std::ops::ControlFlow::{Break, Continue};
use crate::gametime::{GameTimer, TimeMessage, TimeReceived};
use crate::client::tcpinput::TcpInput;
use crate::interface::{EventSender, GameFactoryTrait, GameTrait, Sender};
use crate::client::tcpoutput::TcpOutput;
use crate::messaging::{InitialInformation, InputMessage};
use crate::gamemanager::{Manager, ManagerEvent, RenderReceiverMessage};
use log::trace;
use commons::factory::FactoryTrait;
use commons::net::UdpSocketTrait;
use crate::client::clientcore::ClientCoreEvent::{OnInitialInformation, OnInputEvent, GameTimerTick, RemoteTimeMessageEvent};
use crate::client::clientgametimeobserver::ClientGameTimerObserver;
use crate::client::clientmanagerobserver::ClientManagerObserver;
use crate::client::udpoutput::{UdpOutput, UdpOutputEvent};
use crate::client::udpinput::UdpInput;
use commons::threading::AsyncJoin;
use commons::threading::channel::{ReceiveMetaData, SenderTrait};
use commons::threading::eventhandling::{ChannelEvent, EventHandleResult, EventHandlerTrait, EventSenderTrait};
use commons::threading::eventhandling::WaitOrTryForNextEvent::{TryForNextEvent, WaitForNextEvent};

pub enum ClientCoreEvent<GameFactory: GameFactoryTrait> {
    OnInitialInformation(InitialInformation<GameFactory::Game>),
    OnInputEvent(<GameFactory::Game as GameTrait>::ClientInputEvent),
    GameTimerTick,
    RemoteTimeMessageEvent(TimeReceived<TimeMessage>)
}

//TODO: make this an enum, get rid of all the options
pub struct ClientCore<GameFactory: GameFactoryTrait> {
    factory: GameFactory::Factory,
    sender: EventSender<GameFactory, ClientCoreEvent<GameFactory>>,
    server_ip: Ipv4Addr,
    input_event_handler: <GameFactory::Game as GameTrait>::ClientInputEventHandler,
    manager_sender: EventSender<GameFactory, ManagerEvent<GameFactory::Game>>,
    game_timer: Option<GameTimer<GameFactory::Factory, ClientGameTimerObserver<GameFactory>>>,
    udp_input_sender_option: Option<EventSender<GameFactory, ()>>,
    udp_output_sender_option: Option<EventSender<GameFactory, UdpOutputEvent<GameFactory::Game>>>,
    tcp_input_sender: EventSender<GameFactory, ()>,
    tcp_output_sender: EventSender<GameFactory, ()>,
    render_receiver_sender: Sender<GameFactory, RenderReceiverMessage<GameFactory::Game>>,
    initial_information: Option<InitialInformation<GameFactory::Game>>,
    last_time_message: Option<TimeMessage>
}

impl<GameFactory: GameFactoryTrait> ClientCore<GameFactory> {

    pub fn new(
        factory: GameFactory::Factory,
        server_ip: Ipv4Addr,
        sender: EventSender<GameFactory, ClientCoreEvent<GameFactory>>,
        render_receiver_sender: Sender<GameFactory, RenderReceiverMessage<GameFactory::Game>>
    ) -> Self {

        let client_manager_observer = ClientManagerObserver::<GameFactory>::new(
            factory.clone(),
            render_receiver_sender.clone()
        );

        let manager = Manager::new(
            factory.clone(),
            client_manager_observer
        );

        let manager_sender = factory.new_thread_builder()
            .name("ClientManager")
            .spawn_event_handler(
                manager,
                AsyncJoin::log_async_join)
            .unwrap();

        let socket_addr_v4 = SocketAddrV4::new(server_ip.clone(), GameFactory::Game::TCP_PORT);
        let socket_addr = SocketAddr::from(socket_addr_v4);

        let (tcp_sender, tcp_receiver) = factory.connect_tcp(socket_addr).unwrap();

        let tcp_input = TcpInput::<GameFactory>::new(
            factory.clone(),
            manager_sender.clone(),
            sender.clone(),
            render_receiver_sender.clone());

        let tcp_input_sender = factory.new_thread_builder()
            .name("ClientTcpInput")
            .spawn_tcp_reader(tcp_receiver, tcp_input, AsyncJoin::log_async_join)
            .unwrap();

        let tcp_output_sender = factory.new_thread_builder()
            .name("ClientTcpOutput")
            .spawn_event_handler(TcpOutput::<GameFactory>::new(tcp_sender), AsyncJoin::log_async_join)
            .unwrap();

        return Self {
            factory,
            sender,
            server_ip,
            input_event_handler: GameFactory::Game::new_input_event_handler(),
            manager_sender,
            game_timer: None,
            udp_input_sender_option: None,
            udp_output_sender_option: None,
            tcp_input_sender,
            tcp_output_sender,
            render_receiver_sender,
            initial_information: None,
            last_time_message: None
        }
    }

    fn on_input_event(mut self, input_event: <GameFactory::Game as GameTrait>::ClientInputEvent) -> EventHandleResult<Self> {

        if self.last_time_message.is_some() &&
            self.initial_information.is_some() {

            GameFactory::Game::handle_input_event(&mut self.input_event_handler, input_event);
        }

        return Continue(TryForNextEvent(self));
    }

    fn on_initial_information(mut self, initial_information: InitialInformation<GameFactory::Game>) -> EventHandleResult<Self> {

        let client_game_time_observer = ClientGameTimerObserver::new(self.factory.clone(), self.sender.clone());

        let game_timer = GameTimer::new(
            self.factory.clone(),
            *initial_information.get_server_config(),
            GameFactory::Game::CLOCK_AVERAGE_SIZE,
            client_game_time_observer
        );

        let server_udp_socket_addr = SocketAddr::V4(SocketAddrV4::new(self.server_ip, GameFactory::Game::UDP_PORT));

        let addr = Ipv4Addr::new(127, 0, 0, 1);
        let socket_addr = SocketAddr::V4(SocketAddrV4::new(addr, 0));

        let udp_socket = self.factory.bind_udp_socket(socket_addr).unwrap();

        let udp_input_join_handle = self.factory.new_thread_builder()
            .name("ClientUdpInput")
            .spawn_udp_reader(
                udp_socket.try_clone().unwrap(),
                UdpInput::<GameFactory>::new(
                    self.factory.clone(),
                    self.sender.clone(),
                    self.manager_sender.clone()
                ).unwrap(),
                AsyncJoin::log_async_join)
            .unwrap();

        let udp_output_builder = self.factory.new_thread_builder()
            .name("ClientUdpOutput")
            .build_channel_for_event_handler::<UdpOutput<GameFactory>>();

        //TODO: unwrap after try_clone is not good
        let udp_output_join_handle = udp_output_builder.spawn_event_handler(
            UdpOutput::<GameFactory>::new(server_udp_socket_addr, udp_socket.try_clone().unwrap(), initial_information.clone()),
            AsyncJoin::log_async_join
        ).unwrap();

        self.initial_information = Some(initial_information);
        self.game_timer = Some(game_timer);
        self.udp_input_sender_option = Some(udp_input_join_handle);
        self.udp_output_sender_option = Some(udp_output_join_handle);

        return Continue(TryForNextEvent(self));
    }

    fn on_game_timer_tick(mut self) -> EventHandleResult<Self> {

        let time_message = self.game_timer.as_ref().unwrap().create_timer_message();

        trace!("TimeMessage step_index: {:?}", time_message.get_step());

        //TODO: check if this tick is really the next tick?
        //TODO: log a warn if a tick is missed or out of order
        if self.last_time_message.is_some() &&
            self.initial_information.is_some() {

            let last_time_message = self.last_time_message.as_ref().unwrap();
            let initial_information = self.initial_information.as_ref().unwrap();

            if time_message.get_step() > last_time_message.get_step() {
                let message = InputMessage::<GameFactory::Game>::new(
                    //TODO: message or last message?
                    //TODO: define strict and consistent rules for how real time relates to ticks, input deadlines and display states
                    time_message.get_step(),
                    initial_information.get_player_index(),
                    GameFactory::Game::get_input(& mut self.input_event_handler)
                );

                self.manager_sender.send_event(ManagerEvent::InputEvent(message.clone())).unwrap();
                self.udp_output_sender_option.as_ref().unwrap().send_event(UdpOutputEvent::InputMessageEvent(message)).unwrap();

                let client_drop_time = time_message.get_scheduled_time().subtract(GameFactory::Game::GRACE_PERIOD * 2.0);
                let drop_step = time_message.get_step_from_actual_time(client_drop_time).ceil() as usize;

                self.manager_sender.send_event(ManagerEvent::DropStepsBeforeEvent(drop_step)).unwrap();
                //TODO: message or last message or next?
                //TODO: define strict and consistent rules for how real time relates to ticks, input deadlines and display states
                self.manager_sender.send_event(ManagerEvent::SetRequestedStepEvent(time_message.get_step() + 1)).unwrap();
            }
        }

        self.render_receiver_sender.send(RenderReceiverMessage::TimeMessage(time_message.clone())).unwrap();

        self.last_time_message = Some(time_message);

        return Continue(TryForNextEvent(self));
    }

    fn on_remote_timer_message(mut self, time_message: TimeReceived<TimeMessage>) -> EventHandleResult<Self> {
        self.game_timer.as_mut().unwrap().on_time_message(time_message);
        return Continue(TryForNextEvent(self));
    }
}

impl<GameFactory: GameFactoryTrait> EventHandlerTrait for ClientCore<GameFactory> {
    type Event = ClientCoreEvent<GameFactory>;
    type ThreadReturn = ();

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> EventHandleResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(_, OnInitialInformation(initial_information)) => self.on_initial_information(initial_information),
            ChannelEvent::ReceivedEvent(_, OnInputEvent(client_input_event)) => self.on_input_event(client_input_event),
            ChannelEvent::ReceivedEvent(_, GameTimerTick) => self.on_game_timer_tick(),
            ChannelEvent::ReceivedEvent(_, RemoteTimeMessageEvent(time_message)) => self.on_remote_timer_message(time_message),
            ChannelEvent::Timeout => Continue(WaitForNextEvent(self)),
            ChannelEvent::ChannelEmpty => Continue(WaitForNextEvent(self)),
            ChannelEvent::ChannelDisconnected => Break(())
        }
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn { () }
}
