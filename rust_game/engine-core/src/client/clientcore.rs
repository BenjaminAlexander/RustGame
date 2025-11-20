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
    GameTrait,
    InitialInformation,
    RenderReceiverMessage,
};
use crate::messaging::InputMessage;
use commons::real_time::net::tcp::TcpReadHandlerBuilder;
use commons::real_time::net::udp::UdpReadHandlerBuilder;
use commons::real_time::timer_service::{IdleTimerService, TimerService};
use commons::real_time::{
    EventHandleResult,
    EventHandlerBuilder,
    EventHandlerStopper,
    EventSender,
    Factory,
    HandleEvent,
    ReceiveMetaData,
    Sender,
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

pub enum ClientCoreEvent<Game: GameTrait> {
    OnInitialInformation(InitialInformation<Game>),
    OnInputEvent(Game::ClientInputEvent),
    GameTimerTick,
    RemoteTimeMessageEvent(TimeReceived<TimeMessage>),
}

pub struct ClientCore<Game: GameTrait> {
    factory: Factory,
    sender: EventSender<ClientCoreEvent<Game>>,
    server_ip: Ipv4Addr,
    tcp_input_sender: EventHandlerStopper,
    tcp_output_sender: EventSender<()>,
    render_receiver_sender: Sender<RenderReceiverMessage<Game>>,
    running_state: Option<RunningState<Game>>,
}

//TODO: don't start client core before hello
struct RunningState<Game: GameTrait> {
    manager_sender: EventSender<ManagerEvent<Game>>,
    input_event_handler: Game::ClientInputEventHandler,
    timer_service: TimerService<(), ClientGameTimerObserver<Game>>,
    game_timer: GameTimer,
    udp_input_sender: EventHandlerStopper,
    udp_output_sender: EventSender<UdpOutputEvent<Game>>,
    initial_information: InitialInformation<Game>,
    last_time_message: Option<TimeMessage>,
}

impl<Game: GameTrait> ClientCore<Game> {
    pub fn new(
        factory: Factory,
        server_ip: Ipv4Addr,
        sender: EventSender<ClientCoreEvent<Game>>,
        render_receiver_sender: Sender<RenderReceiverMessage<Game>>,
    ) -> Self {
        let socket_addr_v4 = SocketAddrV4::new(server_ip.clone(), Game::TCP_PORT);
        let socket_addr = SocketAddr::from(socket_addr_v4);

        let (tcp_sender, tcp_receiver) = factory.connect_tcp(socket_addr).unwrap();

        let tcp_input = TcpInput::<Game>::new(sender.clone(), render_receiver_sender.clone());

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

        return Self {
            factory,
            sender,
            server_ip,
            tcp_input_sender,
            tcp_output_sender,
            render_receiver_sender,
            running_state: None,
        };
    }

    fn on_initial_information(
        &mut self,
        initial_information: InitialInformation<Game>,
    ) -> EventHandleResult {
        if self.running_state.is_some() {
            warn!("Received a hello from the server after the client has already received a hello");
            return EventHandleResult::TryForNextEvent;
        }

        //TODO: maybe consolidate building of the manager into its own method
        let client_manager_observer =
            ClientManagerObserver::<Game>::new(self.render_receiver_sender.clone());

        let manager = Manager::new(
            self.factory.get_time_source().clone(),
            client_manager_observer,
            initial_information.clone(),
        );

        let manager_sender =
            EventHandlerBuilder::new_thread(&self.factory, "ClientManager".to_string(), manager)
                .unwrap();

        let mut idle_timer_service = IdleTimerService::new();

        let game_timer = GameTimer::new(
            &self.factory,
            &mut idle_timer_service,
            *initial_information
                .get_server_config()
                .get_game_timer_config(),
            Game::CLOCK_AVERAGE_SIZE,
            ClientGameTimerObserver::new(self.sender.clone()),
        );

        let timer_service = idle_timer_service.start(&self.factory).unwrap();

        let server_udp_socket_addr =
            SocketAddr::V4(SocketAddrV4::new(self.server_ip, Game::UDP_PORT));

        let udp_socket = self.factory.bind_udp_ephemeral_port().unwrap();

        let udp_input_sender = UdpReadHandlerBuilder::new_thread(
            &self.factory,
            "ClientUdpInput".to_string(),
            udp_socket.try_clone().unwrap(),
            UdpInput::<Game>::new(
                self.factory.get_time_source().clone(),
                self.sender.clone(),
                manager_sender.clone(),
            )
            .unwrap(),
        )
        .unwrap();

        //TODO: unwrap after try_clone is not good
        let udp_output_sender = EventHandlerBuilder::new_thread(
            &self.factory,
            "ClientUdpOutput".to_string(),
            UdpOutput::<Game>::new(
                server_udp_socket_addr,
                udp_socket.try_clone().unwrap(),
                initial_information.clone(),
            ),
        )
        .unwrap();

        self.running_state = Some(RunningState {
            manager_sender,
            input_event_handler: Game::new_input_event_handler(),
            timer_service,
            game_timer,
            udp_input_sender,
            udp_output_sender,
            initial_information,
            last_time_message: None,
        });

        return EventHandleResult::TryForNextEvent;
    }

    fn on_input_event(&mut self, input_event: Game::ClientInputEvent) -> EventHandleResult {
        if let Some(ref mut running_state) = self.running_state {
            if running_state.last_time_message.is_some() {
                Game::handle_input_event(&mut running_state.input_event_handler, input_event);
            }
        }
        //Else: No-op, just discard the input

        return EventHandleResult::TryForNextEvent;
    }

    fn on_game_timer_tick(&mut self) -> EventHandleResult {
        if let Some(ref mut running_state) = self.running_state {
            let time_message = running_state.game_timer.create_timer_message();

            trace!("TimeMessage step_index: {:?}", time_message.get_step());

            //TODO: check if this tick is really the next tick?
            //TODO: log a warn if a tick is missed or out of order
            if running_state.last_time_message.is_some() {
                let last_time_message = running_state.last_time_message.as_ref().unwrap();

                if time_message.get_step() > last_time_message.get_step() {
                    let message = InputMessage::<Game>::new(
                        //TODO: message or last message?
                        //TODO: define strict and consistent rules for how real time relates to ticks, input deadlines and display states
                        time_message.get_step(),
                        running_state.initial_information.get_player_index(),
                        Game::get_input(&mut running_state.input_event_handler),
                    );

                    let send_result = running_state
                        .manager_sender
                        .send_event(ManagerEvent::InputEvent(message.clone()));

                    if send_result.is_err() {
                        warn!("Failed to send InputMessage to Game Manager");
                        return EventHandleResult::StopThread;
                    }

                    let send_result = running_state
                        .udp_output_sender
                        .send_event(UdpOutputEvent::InputMessageEvent(message));

                    if send_result.is_err() {
                        warn!("Failed to send InputMessage to Udp Output");
                        return EventHandleResult::StopThread;
                    }

                    //TODO: don't calculate this every time
                    let client_drop_time = time_message
                        .get_scheduled_time()
                        .sub(&Game::GRACE_PERIOD.mul_f64(2.0));

                    let drop_step = time_message
                        .get_step_from_actual_time(client_drop_time)
                        .ceil() as usize;

                    let send_result = running_state
                        .manager_sender
                        .send_event(ManagerEvent::DropStepsBeforeEvent(drop_step));

                    if send_result.is_err() {
                        warn!("Failed to send Drop Steps to Game Manager");
                        return EventHandleResult::StopThread;
                    }

                    //TODO: message or last message or next?
                    //TODO: define strict and consistent rules for how real time relates to ticks, input deadlines and display states
                    let send_result = running_state.manager_sender.send_event(
                        ManagerEvent::SetRequestedStepEvent(time_message.get_step() + 1),
                    );

                    if send_result.is_err() {
                        warn!("Failed to send Request Step to Game Manager");
                        return EventHandleResult::StopThread;
                    }
                }
            }

            let send_result = self
                .render_receiver_sender
                .send(RenderReceiverMessage::TimeMessage(time_message.clone()));

            if send_result.is_err() {
                warn!("Failed to send TimeMessage Step to Render Receiver");
                return EventHandleResult::StopThread;
            }

            running_state.last_time_message = Some(time_message);
        } else {
            warn!("Received a game timer tick while waiting for the hello from the server")
        }

        return EventHandleResult::TryForNextEvent;
    }

    fn on_remote_timer_message(
        &mut self,
        time_message: TimeReceived<TimeMessage>,
    ) -> EventHandleResult {
        if let Some(ref mut running_state) = self.running_state {
            running_state
                .game_timer
                .on_remote_timer_message(&running_state.timer_service, time_message)
                .unwrap();
        } else {
            warn!("Received a remote timer message while waiting for the hello from the server")
        }

        return EventHandleResult::TryForNextEvent;
    }
}

impl<Game: GameTrait> HandleEvent for ClientCore<Game> {
    type Event = ClientCoreEvent<Game>;
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, event: Self::Event) -> EventHandleResult {
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

    fn on_stop_self(self) -> Self::ThreadReturn {
        return ();
    }
}
