use crate::aggregate_input::AggregateInput;
use crate::client::clientgametimeobserver::ClientGameTimerObserver;
use crate::client::clientmanagerobserver::ClientManagerObserver;
use crate::client::tcpinput::TcpInput;
use crate::client::tcpoutput::TcpOutput;
use crate::client::udpinput::UdpInput;
use crate::client::udpoutput::{
    UdpOutput,
    UdpOutputEvent,
};
use crate::frame_manager::FrameManager;
use crate::game_time::{
    CompletedPing,
    FrameIndex,
    GameTimerScheduler,
};
use crate::interface::{
    GameTrait,
    InitialInformation,
};
use crate::messaging::ToServerInputMessage;
use crate::state_channel::StateSender;
use commons::real_time::net::tcp::TcpReadHandlerBuilder;
use commons::real_time::net::udp::UdpReadHandlerBuilder;
use commons::real_time::timer_service::{
    IdleTimerService,
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
use log::{
    trace,
    warn,
};
use std::net::{
    Ipv4Addr,
    SocketAddr,
    SocketAddrV4,
};

pub enum ClientCoreEvent<Game: GameTrait> {
    OnInitialInformation(InitialInformation<Game>),
    OnInputEvent(Game::ClientInputEvent),
    GameTimerTick,
    CompletedPing(CompletedPing),
}

pub struct ClientCore<Game: GameTrait> {
    factory: Factory,
    sender: EventSender<ClientCoreEvent<Game>>,
    server_ip: Ipv4Addr,
    tcp_input_sender: EventHandlerStopper,
    tcp_output_sender: EventSender<()>,
    state_sender: StateSender<Game>,
    running_state: Option<RunningState<Game>>,
}

//TODO: don't start client core before hello
struct RunningState<Game: GameTrait> {
    frame_manager: FrameManager<Game>,
    input_aggregator: Game::InputAggregator,
    timer_service: TimerService<(), ClientGameTimerObserver<Game>>,
    game_timer: GameTimerScheduler,
    udp_input_sender: EventHandlerStopper,
    udp_output_sender: EventSender<UdpOutputEvent<Game>>,
    initial_information: InitialInformation<Game>,
    input_grace_period_frames: usize,
}

impl<Game: GameTrait> ClientCore<Game> {
    pub fn new(
        factory: Factory,
        server_ip: Ipv4Addr,
        sender: EventSender<ClientCoreEvent<Game>>,
        state_sender: StateSender<Game>
    ) -> Self {
        let socket_addr_v4 = SocketAddrV4::new(server_ip.clone(), Game::TCP_PORT);
        let socket_addr = SocketAddr::from(socket_addr_v4);

        let (tcp_sender, tcp_receiver) = factory.connect_tcp(socket_addr).unwrap();

        let tcp_input = TcpInput::<Game>::new(sender.clone(), state_sender.clone());

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
            state_sender,
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
            ClientManagerObserver::<Game>::new(self.state_sender.clone());

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
                frame_manager.clone(),
            )
            .unwrap(),
        )
        .unwrap();

        //TODO: unwrap after try_clone is not good
        let udp_output_sender = EventHandlerBuilder::new_thread(
            &self.factory,
            "ClientUdpOutput".to_string(),
            UdpOutput::<Game>::new(
                self.factory.get_time_source().clone(),
                server_udp_socket_addr,
                udp_socket.try_clone().unwrap(),
                initial_information.clone(),
            ),
        )
        .unwrap();

        let input_grace_period_frames = initial_information
            .get_server_config()
            .get_frame_duration()
            .to_frame_count(&Game::GRACE_PERIOD.mul_f64(2.0))
            as usize;

        self.running_state = Some(RunningState {
            frame_manager,
            input_aggregator: Game::InputAggregator::new(),
            timer_service,
            game_timer,
            udp_input_sender,
            udp_output_sender,
            initial_information,
            input_grace_period_frames,
        });

        // TODO: this causes the first client ping to be requested.  If the first
        // ping is dropped (its udp), then the client clock will never start.
        // There should probably be some retry logic for this.
        return self.send_new_frame_index(FrameIndex::zero());
    }

    fn on_input_event(&mut self, input_event: Game::ClientInputEvent) -> EventHandleResult {
        if let Some(ref mut running_state) = self.running_state {
            running_state.input_aggregator.aggregate_input_event(input_event);
        }
        //Else: No-op, just discard the input

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

    fn send_new_frame_index(&mut self, frame_index: FrameIndex) -> EventHandleResult {
        if let Some(ref mut running_state) = self.running_state {
            trace!("TimeMessage step_index: {:?}", frame_index);

            let message = ToServerInputMessage::<Game>::new(
                //TODO: message or last message?
                //TODO: define strict and consistent rules for how real time relates to ticks, input deadlines and display states
                frame_index,
                running_state.initial_information.get_player_index(),
                running_state.input_aggregator.peak_input(),
            );

            running_state.input_aggregator.reset_for_new_frame();

            let send_result = running_state.frame_manager.insert_input(
                message.get_frame_index(),
                message.get_player_index(),
                message.get_input().clone(),
                false,
            );

            if send_result.is_err() {
                warn!("Failed to send InputMessage to Game Manager");
                return EventHandleResult::StopThread;
            }

            let send_result = running_state
                .udp_output_sender
                .send_event(UdpOutputEvent::FrameIndex(frame_index));

            if send_result.is_err() {
                warn!("Failed to send InputMessage to Udp Output");
                return EventHandleResult::StopThread;
            }

            let send_result = running_state
                .udp_output_sender
                .send_event(UdpOutputEvent::InputMessageEvent(message));

            if send_result.is_err() {
                warn!("Failed to send InputMessage to Udp Output");
                return EventHandleResult::StopThread;
            }

            let send_result = running_state.frame_manager.advance_frame_index(frame_index);

            if send_result.is_err() {
                warn!("Failed to send FrameIndex to Game Manager");
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
            ClientCoreEvent::CompletedPing(completed_ping) => {
                self.on_completed_ping(completed_ping)
            }
        };
    }

    fn on_stop_self(self) -> Self::ThreadReturn {
        return ();
    }
}
