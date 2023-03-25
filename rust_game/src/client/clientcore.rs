use std::net::{Ipv4Addr, SocketAddrV4, SocketAddr, TcpStream, UdpSocket};
use std::ops::ControlFlow::{Break, Continue};
use crate::gametime::{GameTimer, TimeMessage, TimeReceived};
use crate::client::tcpinput::TcpInput;
use crate::interface::{GameFactoryTrait, GameTrait};
use crate::client::tcpoutput::TcpOutput;
use crate::messaging::{InitialInformation, InputMessage};
use crate::gamemanager::{Manager, ManagerEvent, RenderReceiverMessage};
use log::{trace};
use commons::factory::FactoryTrait;
use crate::client::clientcore::ClientCoreEvent::{Connect, OnInitialInformation, OnInputEvent, GameTimerTick, RemoteTimeMessageEvent};
use crate::client::clientgametimeobserver::ClientGameTimerObserver;
use crate::client::clientmanagerobserver::ClientManagerObserver;
use crate::client::udpoutput::{UdpOutput, UdpOutputEvent};
use crate::client::udpinput::UdpInput;
use commons::threading::{ThreadBuilder, AsyncJoin};
use commons::threading::channel::{ReceiveMetaData, SenderTrait};
use commons::threading::eventhandling;
use commons::threading::eventhandling::{ChannelEvent, ChannelEventResult, EventHandlerTrait, EventSenderTrait};
use commons::threading::eventhandling::WaitOrTryForNextEvent::{TryForNextEvent, WaitForNextEvent};

pub enum ClientCoreEvent<GameFactory: GameFactoryTrait> {
    Connect(<GameFactory::Factory as FactoryTrait>::Sender<RenderReceiverMessage<GameFactory::Game>>),
    OnInitialInformation(InitialInformation<GameFactory::Game>),
    OnInputEvent(<GameFactory::Game as GameTrait>::ClientInputEvent),
    GameTimerTick,
    RemoteTimeMessageEvent(TimeReceived<TimeMessage>)
}

//TODO: make this an enum, get rid of all the options
pub struct ClientCore<GameFactory: GameFactoryTrait> {
    factory: GameFactory::Factory,
    sender: eventhandling::Sender<GameFactory::Factory, ClientCoreEvent<GameFactory>>,
    server_ip: Ipv4Addr,
    input_event_handler: <GameFactory::Game as GameTrait>::ClientInputEventHandler,
    manager_sender_option: Option<eventhandling::Sender<GameFactory::Factory, ManagerEvent<GameFactory::Game>>>,
    game_timer: Option<GameTimer<GameFactory::Factory, ClientGameTimerObserver<GameFactory>>>,
    udp_input_sender_option: Option<eventhandling::Sender<GameFactory::Factory, ()>>,
    udp_output_sender_option: Option<eventhandling::Sender<GameFactory::Factory, UdpOutputEvent<GameFactory::Game>>>,
    tcp_input_sender_option: Option<eventhandling::Sender<GameFactory::Factory, ()>>,
    tcp_output_sender_option: Option<eventhandling::Sender<GameFactory::Factory, ()>>,
    render_receiver_sender: Option<<GameFactory::Factory as FactoryTrait>::Sender<RenderReceiverMessage<GameFactory::Game>>>,
    initial_information: Option<InitialInformation<GameFactory::Game>>,
    last_time_message: Option<TimeMessage>
}

impl<GameFactory: GameFactoryTrait> ClientCore<GameFactory> {

    pub fn new(factory: GameFactory::Factory, server_ip: Ipv4Addr, sender: eventhandling::Sender<GameFactory::Factory, ClientCoreEvent<GameFactory>>) -> Self {

        ClientCore {
            factory,
            sender,
            server_ip,
            input_event_handler: GameFactory::Game::new_input_event_handler(),
            manager_sender_option: None,
            game_timer: None,
            udp_input_sender_option: None,
            udp_output_sender_option: None,
            tcp_input_sender_option: None,
            tcp_output_sender_option: None,
            render_receiver_sender: None,
            initial_information: None,
            last_time_message: None
        }
    }

    fn connect(mut self, render_receiver_sender: <GameFactory::Factory as FactoryTrait>::Sender<RenderReceiverMessage<GameFactory::Game>>) -> ChannelEventResult<Self> {

        let socket_addr_v4 = SocketAddrV4::new(self.server_ip, GameFactory::Game::TCP_PORT);
        let socket_addr = SocketAddr::from(socket_addr_v4);
        let tcp_stream = TcpStream::connect(socket_addr).unwrap();

        let manager_sender = ThreadBuilder::new(self.factory.clone())
            .name("ClientManager")
            .spawn_event_handler(Manager::new(self.factory.clone(), ClientManagerObserver::<GameFactory>::new(self.factory.clone(), render_receiver_sender.clone())), AsyncJoin::log_async_join)
            .unwrap();

        let tcp_input_sender = ThreadBuilder::new(self.factory.clone())
            .name("ClientTcpInput")
            .spawn_listener(TcpInput::<GameFactory>::new(
                self.factory.clone(),
                manager_sender.clone(),
                self.sender.clone(),
                render_receiver_sender.clone(),
                &tcp_stream).unwrap(), AsyncJoin::log_async_join)
            .unwrap();

        let tcp_output_join_handle = ThreadBuilder::new(self.factory.clone())
            .name("ClientTcpOutput")
            .spawn_event_handler(TcpOutput::new(&tcp_stream).unwrap(), AsyncJoin::log_async_join)
            .unwrap();

        self.render_receiver_sender = Some(render_receiver_sender);
        self.manager_sender_option = Some(manager_sender);
        self.tcp_output_sender_option = Some(tcp_output_join_handle);
        self.tcp_input_sender_option = Some(tcp_input_sender);

        return Continue(TryForNextEvent(self));
    }

    fn on_input_event(mut self, input_event: <GameFactory::Game as GameTrait>::ClientInputEvent) -> ChannelEventResult<Self> {

        if self.manager_sender_option.is_some() &&
            self.last_time_message.is_some() &&
            self.initial_information.is_some() {

            GameFactory::Game::handle_input_event(&mut self.input_event_handler, input_event);
        }

        return Continue(TryForNextEvent(self));
    }

    fn on_initial_information(mut self, initial_information: InitialInformation<GameFactory::Game>) -> ChannelEventResult<Self> {

        let client_game_time_observer = ClientGameTimerObserver::new(self.factory.clone(), self.sender.clone());

        let game_timer = GameTimer::new(
            self.factory.clone(),
            *initial_information.get_server_config(),
            GameFactory::Game::CLOCK_AVERAGE_SIZE,
            client_game_time_observer
        );

        let server_udp_socket_addr_v4 = SocketAddrV4::new(self.server_ip, GameFactory::Game::UDP_PORT);

        //TODO: pass in as a parameter
        let udp_socket = UdpSocket::bind("127.0.0.1:0").unwrap();

        let udp_input_join_handle = ThreadBuilder::new(self.factory.clone())
            .name("ClientUdpInput")
            .spawn_listener(
                UdpInput::<GameFactory>::new(
                    self.factory.clone(),
                    server_udp_socket_addr_v4,
                    &udp_socket,
                    self.sender.clone(),
                    self.manager_sender_option.as_ref().unwrap().clone()
                ).unwrap(),
                AsyncJoin::log_async_join)
            .unwrap();

        let udp_output_builder = ThreadBuilder::new(self.factory.clone())
            .name("ClientUdpOutput")
            .build_channel_for_event_handler::<UdpOutput<GameFactory::Game>>();

        //TODO: unwrap after try_clone is not good
        let udp_output_join_handle = udp_output_builder.spawn_event_handler(
            UdpOutput::<GameFactory::Game>::new(server_udp_socket_addr_v4, udp_socket.try_clone().unwrap(), initial_information.clone()),
            AsyncJoin::log_async_join
        ).unwrap();

        self.initial_information = Some(initial_information);
        self.game_timer = Some(game_timer);
        self.udp_input_sender_option = Some(udp_input_join_handle);
        self.udp_output_sender_option = Some(udp_output_join_handle);

        return Continue(TryForNextEvent(self));
    }

    fn on_game_timer_tick(mut self) -> ChannelEventResult<Self> {

        let time_message = self.game_timer.as_ref().unwrap().create_timer_message();

        trace!("TimeMessage step_index: {:?}", time_message.get_step());

        //TODO: check if this tick is really the next tick?
        //TODO: log a warn if a tick is missed or out of order
        if self.last_time_message.is_some() &&
            self.tcp_output_sender_option.is_some() &&
            self.initial_information.is_some() &&
            self.manager_sender_option.is_some() {

            let manager_sender = self.manager_sender_option.as_ref().unwrap();
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

                manager_sender.send_event(ManagerEvent::InputEvent(message.clone())).unwrap();
                self.udp_output_sender_option.as_ref().unwrap().send_event(UdpOutputEvent::InputMessageEvent(message)).unwrap();

                let client_drop_time = time_message.get_scheduled_time().subtract(GameFactory::Game::GRACE_PERIOD * 2.0);
                let drop_step = time_message.get_step_from_actual_time(client_drop_time).ceil() as usize;

                manager_sender.send_event(ManagerEvent::DropStepsBeforeEvent(drop_step)).unwrap();
                //TODO: message or last message or next?
                //TODO: define strict and consistent rules for how real time relates to ticks, input deadlines and display states
                manager_sender.send_event(ManagerEvent::SetRequestedStepEvent(time_message.get_step() + 1)).unwrap();
            }
        }


        self.render_receiver_sender.as_ref().unwrap().send(RenderReceiverMessage::TimeMessage(time_message.clone())).unwrap();

        self.last_time_message = Some(time_message);

        return Continue(TryForNextEvent(self));
    }

    fn on_remote_timer_message(mut self, time_message: TimeReceived<TimeMessage>) -> ChannelEventResult<Self> {
        self.game_timer.as_mut().unwrap().on_time_message(time_message);
        return Continue(TryForNextEvent(self));
    }
}

impl<GameFactory: GameFactoryTrait> EventHandlerTrait for ClientCore<GameFactory> {
    type Event = ClientCoreEvent<GameFactory>;
    type ThreadReturn = ();

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> ChannelEventResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(_, Connect(render_receiver_sender)) => self.connect(render_receiver_sender),
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