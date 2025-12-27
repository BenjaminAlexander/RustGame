use crate::Play;
use crate::aggregate_input::AggregateInput;
use crate::client::clientmanagerobserver::ClientManagerObserver;
use crate::client::tcp_input::TcpInput;
use crate::client::tcpoutput::TcpOutput;
use crate::client::udp_output::UdpOutput;
use crate::client::udpinput::UdpInput;
use crate::frame_manager::{
    ClientProvidedInput,
    FrameManager,
    InputCoordinates,
    PresentInput,
};
use crate::game_time::{
    CompletedPing,
    FrameIndex,
    GameTimerScheduler,
};
use crate::interface::InitialInformation;
use crate::messaging::ToServerInputMessage;
use crate::state_channel::StateSender;
use commons::real_time::net::udp::UdpReadHandlerBuilder;
use commons::real_time::timer_service::{
    IdleTimerService,
    TimerCallBack,
    TimerService,
};
use commons::real_time::{
    EventHandleResult,
    EventHandlerBuilder,
    EventHandlerStopper,
    EventSender,
    Factory,
    HandleEvent,
    ReceiveMetaData,
};
use commons::time::TimeValue;
use commons::utils::unit_error;
use log::{
    debug,
    trace,
    warn,
};
use std::io::Error;
use std::net::{
    Ipv4Addr,
    SocketAddr,
    SocketAddrV4,
};
use std::str::FromStr;

#[derive(Clone)]
pub struct ClientCore<Game: Play> {
    sender: EventSender<Event<Game>>,
}

impl<Game: Play> ClientCore<Game> {
    pub fn new(factory: Factory, state_sender: StateSender<Game>) -> Result<Self, Error> {
        let builder = EventHandlerBuilder::new(&factory);

        let sender = builder.get_sender().clone();

        let client_core = Self { sender };

        builder.spawn_thread(
            "ClientCore".to_string(),
            EventHandler::<Game>::new(
                factory,
                Ipv4Addr::from_str("127.0.0.1").unwrap(),
                client_core.clone(),
                state_sender,
            )?,
        )?;

        Ok(client_core)
    }

    //TODO: rename as handle_
    pub fn input_event(&self, input_event: Game::InputEvent) -> Result<(), ()> {
        let event = Event::OnInputEvent(input_event);
        self.sender.send_event(event).map_err(unit_error)
    }

    pub fn set_initial_information(
        &self,
        initial_information: InitialInformation<Game>,
    ) -> Result<(), ()> {
        let event = Event::OnInitialInformation(initial_information);
        self.sender.send_event(event).map_err(unit_error)
    }

    pub fn handle_complete_ping(&self, completed_ping: CompletedPing) -> Result<(), ()> {
        let event = Event::CompletedPing(completed_ping);
        self.sender.send_event(event).map_err(unit_error)
    }
}

impl<Game: Play> TimerCallBack for ClientCore<Game> {
    fn tick(&mut self) {
        let send_result = self.sender.send_event(Event::GameTimerTick);
        if send_result.is_err() {
            warn!("Failed to send GameTimerTick to the Core")
        }
    }
}

enum Event<Game: Play> {
    OnInitialInformation(InitialInformation<Game>),
    OnInputEvent(Game::InputEvent),
    GameTimerTick,
    CompletedPing(CompletedPing),
}

struct EventHandler<Game: Play> {
    factory: Factory,
    client_core: ClientCore<Game>,
    server_ip: Ipv4Addr,
    tcp_input: TcpInput,
    tcp_output_sender: EventSender<()>,
    state_sender: StateSender<Game>,
    running_state: Option<RunningState<Game>>,
}

//TODO: don't start client core before hello
struct RunningState<Game: Play> {
    current_frame_index: FrameIndex,
    time_of_current_frame: TimeValue,
    frame_manager: FrameManager<Game>,
    input_aggregator: Game::InputAggregator,
    timer_service: TimerService<(), ClientCore<Game>>,
    game_timer: GameTimerScheduler,
    udp_input_sender: EventHandlerStopper,
    udp_output: UdpOutput<Game>,
    initial_information: InitialInformation<Game>,
}

impl<Game: Play> EventHandler<Game> {
    //TODO: remove
    pub fn new(
        factory: Factory,
        server_ip: Ipv4Addr,
        client_core: ClientCore<Game>,
        state_sender: StateSender<Game>,
    ) -> Result<Self, Error> {
        let socket_addr_v4 = SocketAddrV4::new(server_ip.clone(), Game::TCP_PORT);
        let socket_addr = SocketAddr::from(socket_addr_v4);

        let (tcp_sender, tcp_reader) = factory.connect_tcp(socket_addr).unwrap();

        let tcp_input = TcpInput::new(
            &factory,
            tcp_reader,
            client_core.clone(),
            state_sender.clone(),
        )?;

        let tcp_output_sender = EventHandlerBuilder::new_thread(
            &factory,
            "ClientTcpOutput".to_string(),
            TcpOutput::new(tcp_sender),
        )
        .unwrap();

        return Ok(Self {
            factory,
            client_core,
            server_ip,
            tcp_input,
            tcp_output_sender,
            state_sender,
            running_state: None,
        });
    }

    fn on_initial_information(
        &mut self,
        initial_information: InitialInformation<Game>,
    ) -> EventHandleResult {
        if self.running_state.is_some() {
            warn!("Received a hello from the server after the client has already received a hello");
            return EventHandleResult::TryForNextEvent;
        }

        let server_udp_socket_addr =
            SocketAddr::V4(SocketAddrV4::new(self.server_ip, Game::UDP_PORT));

        let udp_socket = self.factory.bind_udp_ephemeral_port().unwrap();

        //TODO: unwrap after try_clone is not good
        let udp_output = UdpOutput::new(
            &self.factory,
            server_udp_socket_addr,
            udp_socket.try_clone().unwrap(),
            initial_information.clone(),
        )
        .unwrap();

        //TODO: maybe consolidate building of the manager into its own method
        let client_manager_observer =
            ClientManagerObserver::<Game>::new(self.state_sender.clone(), udp_output.clone());

        let frame_manager = FrameManager::new(
            &self.factory,
            client_manager_observer,
            initial_information.clone(),
        )
        .unwrap();

        let mut idle_timer_service = IdleTimerService::new();

        let game_timer = GameTimerScheduler::client_new(
            self.factory.get_time_source().clone(),
            &mut idle_timer_service,
            initial_information.get_server_config(),
            Game::CLOCK_AVERAGE_SIZE,
            self.client_core.clone(),
        );

        let timer_service = idle_timer_service.start(&self.factory).unwrap();

        let udp_input_sender = UdpReadHandlerBuilder::new_thread(
            &self.factory,
            "ClientUdpInput".to_string(),
            udp_socket,
            UdpInput::<Game>::new(
                self.factory.get_time_source().clone(),
                self.client_core.clone(),
                frame_manager.clone(),
            )
            .unwrap(),
        )
        .unwrap();

        self.running_state = Some(RunningState {
            current_frame_index: FrameIndex::zero(),
            time_of_current_frame: self.factory.get_time_source().now(),
            frame_manager,
            input_aggregator: Game::InputAggregator::new(),
            timer_service,
            game_timer,
            udp_input_sender,
            udp_output,
            initial_information,
        });

        // TODO: this causes the first client ping to be requested.  If the first
        // ping is dropped (its udp), then the client clock will never start.
        // There should probably be some retry logic for this.
        return self.send_new_frame_index(FrameIndex::zero());
    }

    fn on_input_event(&mut self, input_event: Game::InputEvent) -> EventHandleResult {
        if let Some(ref mut running_state) = self.running_state {
            let now = self.factory.get_time_source().now();

            // Calculating the fractional time based on the local time of occurance
            // of the current frame ensures the fractional time is always increasing.
            let duration_since_current_frame =
                now.duration_since(&running_state.time_of_current_frame);
            let fractional_frame_duration = (duration_since_current_frame.as_secs_f64()
                / running_state
                    .initial_information
                    .get_server_config()
                    .get_frame_duration()
                    .get_frame_duration()
                    .as_secs_f64())
            .min(1.0)
            .max(0.0);

            let input_option = running_state
                .input_aggregator
                .aggregate_input_event(fractional_frame_duration, input_event);

            if let Some(input) = input_option {
                //TODO: Can this peak code be consolidate with the one from new frame index logic?
                let client_provided_input = ClientProvidedInput::NonAuthoritative(input);

                let input_coordinates = InputCoordinates::new(
                    running_state.current_frame_index,
                    running_state.initial_information.get_player_index(),
                );

                let message = ToServerInputMessage::<Game>::new(
                    input_coordinates,
                    client_provided_input.clone(),
                );

                let send_result = running_state.udp_output.send_input(message.clone());

                if send_result.is_err() {
                    warn!("Failed to send InputMessage to Udp Output");
                    return EventHandleResult::StopThread;
                }

                let send_result = running_state.frame_manager.insert_input(
                    input_coordinates,
                    PresentInput::ClientProvided(client_provided_input),
                );

                if send_result.is_err() {
                    warn!("Failed to send InputMessage to Game Manager");
                    return EventHandleResult::StopThread;
                }
            }
        }

        return EventHandleResult::TryForNextEvent;
    }

    fn on_game_timer_tick(&mut self) -> EventHandleResult {
        let frame_index = match self.running_state {
            Some(ref mut running_state) => match running_state.game_timer.try_advance_frame_index()
            {
                Some(time_message) => time_message,
                None => return EventHandleResult::TryForNextEvent,
            },
            None => {
                warn!("Received a game timer tick while waiting for the hello from the server");
                return EventHandleResult::TryForNextEvent;
            }
        };

        return self.send_new_frame_index(frame_index);
    }

    //TODO: make this a function on the running state
    fn send_new_frame_index(&mut self, frame_index: FrameIndex) -> EventHandleResult {
        if let Some(ref mut running_state) = self.running_state {
            trace!("TimeMessage step_index: {:?}", frame_index);

            running_state.current_frame_index = frame_index;
            running_state.time_of_current_frame = self.factory.get_time_source().now();

            let current_input_coordinates = InputCoordinates::new(
                running_state.current_frame_index,
                running_state.initial_information.get_player_index(),
            );

            let peaked_input = running_state.input_aggregator.start_next_frame();

            let message = ToServerInputMessage::<Game>::new(
                current_input_coordinates,
                ClientProvidedInput::NonAuthoritative(peaked_input.clone()),
            );

            let send_result = running_state.udp_output.send_input(message);

            if send_result.is_err() {
                warn!("Failed to send InputMessage to Udp Output");
                return EventHandleResult::StopThread;
            }

            let send_result = running_state
                .frame_manager
                .client_advance_frame_index(current_input_coordinates, peaked_input);

            if send_result.is_err() {
                warn!("Failed to send InputMessage to Game Manager");
                return EventHandleResult::StopThread;
            }

            let send_result = running_state.udp_output.advance_frame_index(frame_index);

            if send_result.is_err() {
                warn!("Failed to send InputMessage to Udp Output");
                return EventHandleResult::StopThread;
            }
        } else {
            warn!("Tried to send next frame when the core wasn't running")
        }

        return EventHandleResult::TryForNextEvent;
    }

    fn on_completed_ping(&mut self, completed_ping: CompletedPing) -> EventHandleResult {
        if let Some(ref mut running_state) = self.running_state {
            let start_time = match running_state
                .game_timer
                .adjust_client_timer(&running_state.timer_service, completed_ping)
            {
                Ok(start_time) => start_time,
                Err(err) => {
                    warn!("Failed to update GameTime start time: {:?}", err);
                    return EventHandleResult::StopThread;
                }
            };

            if self.state_sender.send_start_time(start_time).is_err() {
                warn!("Failed to send StartTime to Render Receiver");
                return EventHandleResult::StopThread;
            }
        } else {
            warn!("Received a remote timer message while waiting for the hello from the server")
        }

        return EventHandleResult::TryForNextEvent;
    }
}

impl<Game: Play> HandleEvent for EventHandler<Game> {
    type Event = Event<Game>;
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, event: Self::Event) -> EventHandleResult {
        return match event {
            Event::OnInitialInformation(initial_information) => {
                self.on_initial_information(initial_information)
            }
            Event::OnInputEvent(client_input_event) => self.on_input_event(client_input_event),
            Event::GameTimerTick => self.on_game_timer_tick(),
            Event::CompletedPing(completed_ping) => self.on_completed_ping(completed_ping),
        };
    }

    fn on_stop_self(self) {
        if self.tcp_input.stop().is_err() {
            debug!("Error when sending stop to TCP Input");
        }
    }
}
