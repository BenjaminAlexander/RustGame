use crate::client::clientgametimeobserver::ClientGameTimerObserver;
use crate::client::clientmanagerobserver::ClientManagerObserver;
use crate::client::tcpinput::TcpInput;
use crate::client::tcpoutput::TcpOutput;
use crate::client::udpinput::UdpInput;
use crate::client::udpoutput::{
    UdpOutput,
    UdpOutputEvent,
};
use crate::gamemanager::{
    Manager,
    ManagerEvent,
};
use crate::gametime::{
    GameTimer,
    TimeMessage,
    TimeReceived,
};
use crate::interface::{
    GameFactoryTrait,
    GameTrait,
    InitialInformation,
    RenderReceiverMessage,
};
use crate::messaging::InputMessage;
use commons::factory::FactoryTrait;
use commons::net::{
    TcpReadHandlerBuilder,
    UdpReadHandlerBuilder,
};
use commons::threading::channel::{
    ReceiveMetaData,
    Sender,
};
use commons::threading::eventhandling::{
    ChannelEvent,
    EventHandleResult,
    EventHandlerBuilder,
    EventHandlerStopper,
    EventHandlerTrait,
    EventSender,
};
use log::{
    trace,
    warn,
};
use std::net::{
    Ipv4Addr,
    SocketAddr,
    SocketAddrV4,
};
use std::ops::Sub;

pub enum ClientCoreEvent<GameFactory: GameFactoryTrait> {
    OnInitialInformation(InitialInformation<GameFactory::Game>),
    OnInputEvent(<GameFactory::Game as GameTrait>::ClientInputEvent),
    GameTimerTick,
    RemoteTimeMessageEvent(TimeReceived<TimeMessage>),
}

pub struct ClientCore<GameFactory: GameFactoryTrait> {
    state: State<GameFactory>,
}

impl<GameFactory: GameFactoryTrait> ClientCore<GameFactory> {
    pub fn new(
        factory: GameFactory::Factory,
        server_ip: Ipv4Addr,
        sender: EventSender<ClientCoreEvent<GameFactory>>,
        render_receiver_sender: Sender<RenderReceiverMessage<GameFactory::Game>>,
    ) -> Self {
        let client_manager_observer = ClientManagerObserver::<GameFactory>::new(
            factory.clone(),
            render_receiver_sender.clone(),
        );

        let manager = Manager::new(factory.clone(), client_manager_observer);

        let manager_sender = EventHandlerBuilder::new_thread(
            &factory,
            "ClientManager".to_string(),
            manager,
        )
        .unwrap();

        let socket_addr_v4 = SocketAddrV4::new(server_ip.clone(), GameFactory::Game::TCP_PORT);
        let socket_addr = SocketAddr::from(socket_addr_v4);

        let (tcp_sender, tcp_receiver) = factory.connect_tcp(socket_addr).unwrap();

        let tcp_input = TcpInput::<GameFactory>::new(
            factory.clone(),
            manager_sender.clone(),
            sender.clone(),
            render_receiver_sender.clone(),
        );

        let tcp_input_sender = TcpReadHandlerBuilder::new_thread(
            &factory,
            "ClientTcpInput".to_string(),
            tcp_receiver,
            tcp_input,
        )
        .unwrap();

        let tcp_output_sender = EventHandlerBuilder::new_thread(
            &factory,
            "ClientTcpOutput".to_string(),
            TcpOutput::new(tcp_sender),
        )
        .unwrap();

        let state = State::WaitingForHello {
            factory,
            sender,
            server_ip,
            manager_sender,
            tcp_input_sender,
            tcp_output_sender,
            render_receiver_sender,
        };

        return Self { state };
    }
}

impl<GameFactory: GameFactoryTrait> EventHandlerTrait for ClientCore<GameFactory> {
    type Event = ClientCoreEvent<GameFactory>;
    type ThreadReturn = ();

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> EventHandleResult<Self> {
        return match channel_event {
            ChannelEvent::ReceivedEvent(_, core_event) => self.state.on_event(core_event),
            ChannelEvent::Timeout => EventHandleResult::WaitForNextEvent(self),
            ChannelEvent::ChannelEmpty => EventHandleResult::WaitForNextEvent(self),
            ChannelEvent::ChannelDisconnected => EventHandleResult::StopThread(()),
        };
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        return ();
    }
}

enum State<GameFactory: GameFactoryTrait> {
    WaitingForHello {
        factory: GameFactory::Factory,
        sender: EventSender<ClientCoreEvent<GameFactory>>,
        server_ip: Ipv4Addr,
        manager_sender: EventSender<ManagerEvent<GameFactory::Game>>,
        tcp_input_sender: EventHandlerStopper,
        tcp_output_sender: EventSender<()>,
        render_receiver_sender: Sender<RenderReceiverMessage<GameFactory::Game>>,
    },
    Running {
        input_event_handler: <GameFactory::Game as GameTrait>::ClientInputEventHandler,
        manager_sender: EventSender<ManagerEvent<GameFactory::Game>>,
        game_timer: GameTimer<GameFactory::Factory, ClientGameTimerObserver<GameFactory>>,
        udp_input_sender: EventHandlerStopper,
        udp_output_sender: EventSender<UdpOutputEvent<GameFactory::Game>>,
        tcp_input_sender: EventHandlerStopper,
        tcp_output_sender: EventSender<()>,
        render_receiver_sender: Sender<RenderReceiverMessage<GameFactory::Game>>,
        initial_information: InitialInformation<GameFactory::Game>,
        last_time_message: Option<TimeMessage>,
    },
}

impl<GameFactory: GameFactoryTrait> State<GameFactory> {
    fn on_event(
        self,
        event: ClientCoreEvent<GameFactory>,
    ) -> EventHandleResult<ClientCore<GameFactory>> {
        return match event {
            ClientCoreEvent::OnInitialInformation(initial_information) => {
                self.on_initial_information(initial_information)
            }
            ClientCoreEvent::OnInputEvent(client_input_event) => {
                self.on_input_event(client_input_event)
            }
            ClientCoreEvent::GameTimerTick => self.on_game_timer_tick(),
            ClientCoreEvent::RemoteTimeMessageEvent(time_message) => {
                self.on_remote_timer_message(time_message)
            }
        };
    }

    fn try_for_next_event(self) -> EventHandleResult<ClientCore<GameFactory>> {
        let client_core = ClientCore { state: self };

        return EventHandleResult::TryForNextEvent(client_core);
    }

    fn on_initial_information(
        self,
        initial_information: InitialInformation<GameFactory::Game>,
    ) -> EventHandleResult<ClientCore<GameFactory>> {
        match self {
            State::WaitingForHello {
                factory,
                sender,
                server_ip,
                manager_sender,
                tcp_input_sender,
                tcp_output_sender,
                render_receiver_sender,
            } => {
                let client_game_time_observer =
                    ClientGameTimerObserver::new(factory.clone(), sender.clone());

                let game_timer = GameTimer::new(
                    factory.clone(),
                    *initial_information
                        .get_server_config()
                        .get_game_timer_config(),
                    GameFactory::Game::CLOCK_AVERAGE_SIZE,
                    client_game_time_observer,
                );

                let server_udp_socket_addr =
                    SocketAddr::V4(SocketAddrV4::new(server_ip, GameFactory::Game::UDP_PORT));

                let udp_socket = factory.bind_udp_ephemeral_port().unwrap();

                let udp_input_sender = UdpReadHandlerBuilder::new_thread(
                    &factory,
                    "ClientUdpInput".to_string(),
                    udp_socket.try_clone().unwrap(),
                    UdpInput::<GameFactory>::new(
                        factory.clone(),
                        sender.clone(),
                        manager_sender.clone(),
                    )
                    .unwrap(),
                )
                .unwrap();

                //TODO: unwrap after try_clone is not good
                let udp_output_sender = EventHandlerBuilder::new_thread(
                    &factory,
                    "ClientUdpOutput".to_string(),
                    UdpOutput::<GameFactory>::new(
                        server_udp_socket_addr,
                        udp_socket.try_clone().unwrap(),
                        initial_information.clone(),
                    ),
                )
                .unwrap();

                let state = State::Running {
                    input_event_handler: GameFactory::Game::new_input_event_handler(),
                    manager_sender,
                    game_timer,
                    udp_input_sender,
                    udp_output_sender,
                    tcp_input_sender,
                    tcp_output_sender,
                    render_receiver_sender,
                    initial_information,
                    last_time_message: None,
                };

                return EventHandleResult::TryForNextEvent(ClientCore { state });
            }
            State::Running { .. } => {
                warn!("Received a hello from the server after the client has already received a hello");
                return self.try_for_next_event();
            }
        };
    }

    fn on_input_event(
        mut self,
        input_event: <GameFactory::Game as GameTrait>::ClientInputEvent,
    ) -> EventHandleResult<ClientCore<GameFactory>> {
        match &mut self {
            State::WaitingForHello { .. } => { /* No-op: just discard the input */ }
            State::Running {
                ref mut input_event_handler,
                ref last_time_message,
                ..
            } => {
                if last_time_message.is_some() {
                    GameFactory::Game::handle_input_event(input_event_handler, input_event);
                }
            }
        };

        return self.try_for_next_event();
    }

    fn on_game_timer_tick(mut self) -> EventHandleResult<ClientCore<GameFactory>> {
        match &mut self {
            State::WaitingForHello { .. } => {
                warn!("Received a game timer tick while waiting for the hello from the server")
            }
            State::Running {
                ref mut input_event_handler,
                manager_sender,
                game_timer,
                udp_output_sender,
                render_receiver_sender,
                initial_information,
                ref mut last_time_message,
                ..
            } => {
                let time_message = game_timer.create_timer_message();

                trace!("TimeMessage step_index: {:?}", time_message.get_step());

                //TODO: check if this tick is really the next tick?
                //TODO: log a warn if a tick is missed or out of order
                if last_time_message.is_some() {
                    let last_time_message = last_time_message.as_ref().unwrap();

                    if time_message.get_step() > last_time_message.get_step() {
                        let message = InputMessage::<GameFactory::Game>::new(
                            //TODO: message or last message?
                            //TODO: define strict and consistent rules for how real time relates to ticks, input deadlines and display states
                            time_message.get_step(),
                            initial_information.get_player_index(),
                            GameFactory::Game::get_input(input_event_handler),
                        );

                        let send_result =
                            manager_sender.send_event(ManagerEvent::InputEvent(message.clone()));

                        if send_result.is_err() {
                            warn!("Failed to send InputMessage to Game Manager");
                            return EventHandleResult::StopThread(());
                        }

                        let send_result = udp_output_sender
                            .send_event(UdpOutputEvent::InputMessageEvent(message));

                        if send_result.is_err() {
                            warn!("Failed to send InputMessage to Udp Output");
                            return EventHandleResult::StopThread(());
                        }

                        let client_drop_time = time_message
                            .get_scheduled_time()
                            .sub(&GameFactory::Game::GRACE_PERIOD.mul_f64(2.0));

                        let drop_step = time_message
                            .get_step_from_actual_time(client_drop_time)
                            .ceil() as usize;

                        let send_result = manager_sender
                            .send_event(ManagerEvent::DropStepsBeforeEvent(drop_step));

                        if send_result.is_err() {
                            warn!("Failed to send Drop Steps to Game Manager");
                            return EventHandleResult::StopThread(());
                        }

                        //TODO: message or last message or next?
                        //TODO: define strict and consistent rules for how real time relates to ticks, input deadlines and display states
                        let send_result = manager_sender.send_event(
                            ManagerEvent::SetRequestedStepEvent(time_message.get_step() + 1),
                        );

                        if send_result.is_err() {
                            warn!("Failed to send Request Step to Game Manager");
                            return EventHandleResult::StopThread(());
                        }
                    }
                }

                let send_result = render_receiver_sender
                    .send(RenderReceiverMessage::TimeMessage(time_message.clone()));

                if send_result.is_err() {
                    warn!("Failed to send TimeMessage Step to Render Receiver");
                    return EventHandleResult::StopThread(());
                }

                *last_time_message = Some(time_message);
            }
        };

        return self.try_for_next_event();
    }

    fn on_remote_timer_message(
        mut self,
        time_message: TimeReceived<TimeMessage>,
    ) -> EventHandleResult<ClientCore<GameFactory>> {
        match &mut self {
            State::WaitingForHello { .. } => {
                warn!("Received a remote timer message while waiting for the hello from the server")
            }
            State::Running { game_timer, .. } => {
                game_timer.on_remote_timer_message(time_message);
            }
        }

        return self.try_for_next_event();
    }
}
