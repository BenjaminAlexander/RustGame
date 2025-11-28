use crate::game_time::{
    FrameIndex,
    PingRequest,
    PingResponse,
};
use crate::interface::GameTrait;
use crate::messaging::{
    Fragmenter,
    InputMessage,
    ServerInputMessage,
    StateMessage,
    UdpToClientMessage,
};
use crate::server::remoteudppeer::RemoteUdpPeer;
use commons::real_time::net::udp::UdpSocket;
use commons::real_time::net::MAX_UDP_DATAGRAM_SIZE;
use commons::real_time::{
    EventHandleResult, EventHandlerBuilder, EventSender, Factory, HandleEvent, ReceiveMetaData, TimeSource
};
use commons::stats::RollingAverage;
use commons::time::{
    TimeDuration,
    TimeValue,
};
use commons::utils::unit_error;
use log::{
    debug,
    error,
    info,
    warn,
};
use std::io::Error;
use std::marker::PhantomData;
use std::ops::ControlFlow;

#[derive(Clone)]
pub struct UdpOutput<Game: GameTrait> {
    sender: EventSender<UdpOutputEvent<Game>>
}

impl<Game: GameTrait> UdpOutput<Game> {
    pub fn new(factory: Factory, player_index: usize, udp_socket: &UdpSocket) -> Result<Self, Error> {
        let event_handler = UdpOutputEventHandler::<Game>::new(
            factory.get_time_source().clone(),
            player_index,
            &udp_socket,
        )?;

        let sender = EventHandlerBuilder::new_thread(
            &factory,
            format!("ServerUdpOutput-Player-{}", player_index),
            event_handler,
        )?;

        Ok(Self{sender})
    }

    pub fn send_ping_response(&self, time_received: TimeValue, ping_request: PingRequest) -> Result<(), ()> {
        let event = UdpOutputEvent::PingRequest { 
            time_received, 
            ping_request 
        };

        self.sender.send_event(event).map_err(unit_error)
    }

    pub fn set_remote_peer(&self, remote_udp_peer: RemoteUdpPeer) -> Result<(), ()> {
        let event = UdpOutputEvent::RemotePeer(remote_udp_peer);
        self.sender.send_event(event).map_err(unit_error)
    }

    pub fn send_input_message(&self, input_message: InputMessage<Game>) -> Result<(), ()> {
        let event = UdpOutputEvent::SendInputMessage(input_message);
        self.sender.send_event(event).map_err(unit_error)
    }

    pub fn send_server_input_message(&self, server_input_message: ServerInputMessage<Game>) -> Result<(), ()> {
        let event = UdpOutputEvent::SendServerInputMessage(server_input_message);
        self.sender.send_event(event).map_err(unit_error)
    }

    pub fn send_completed_step(&self, step_message: StateMessage<Game>) -> Result<(), ()> {
        let event = UdpOutputEvent::SendCompletedStep(step_message);
        self.sender.send_event(event).map_err(unit_error)
    }
}


// TODO: make private
pub enum UdpOutputEvent<Game: GameTrait> {
    RemotePeer(RemoteUdpPeer),
    PingRequest {
        time_received: TimeValue,
        ping_request: PingRequest,
    },
    SendInputMessage(InputMessage<Game>),
    SendServerInputMessage(ServerInputMessage<Game>),
    SendCompletedStep(StateMessage<Game>),
}

// TODO: make private
pub struct UdpOutputEventHandler<Game: GameTrait> {
    time_source: TimeSource,
    player_index: usize,
    socket: UdpSocket,
    remote_peer: Option<RemoteUdpPeer>,
    fragmenter: Fragmenter,
    last_state_sequence: Option<FrameIndex>,
    phantom: PhantomData<Game>,

    //metrics
    time_in_queue_rolling_average: RollingAverage,
    time_of_last_state_send: TimeValue,
    time_of_last_input_send: TimeValue,
    time_of_last_server_input_send: TimeValue,
}

impl<Game: GameTrait> UdpOutputEventHandler<Game> {
    pub fn new(
        time_source: TimeSource,
        player_index: usize,
        socket: &UdpSocket,
    ) -> Result<Self, Error> {
        let now = time_source.now();

        Ok(UdpOutputEventHandler {
            player_index,
            remote_peer: None,
            //TODO: move clone outside
            socket: socket.try_clone()?,
            //TODO: make max datagram size more configurable
            fragmenter: Fragmenter::new(MAX_UDP_DATAGRAM_SIZE),
            last_state_sequence: None,
            phantom: PhantomData,

            //metrics
            time_in_queue_rolling_average: RollingAverage::new(100),
            time_of_last_state_send: now,
            time_of_last_input_send: now,
            time_of_last_server_input_send: now,

            time_source,
        })
    }

    fn on_remote_peer(&mut self, remote_peer: RemoteUdpPeer) -> EventHandleResult {
        //TODO: could this be checked before calling udpoutput?
        if self.player_index == remote_peer.get_player_index() {
            info!("Setting remote peer: {:?}", remote_peer);
            self.remote_peer = Some(remote_peer);
        }

        return EventHandleResult::TryForNextEvent;
    }

    fn on_completed_step(
        &mut self,
        receive_meta_data: ReceiveMetaData,
        state_message: StateMessage<Game>,
    ) -> EventHandleResult {
        let time_in_queue = receive_meta_data.get_send_meta_data().get_time_sent();

        if self.last_state_sequence.is_none()
            || self.last_state_sequence.as_ref().unwrap() <= &state_message.get_frame_index()
        {
            self.last_state_sequence = Some(state_message.get_frame_index());
            self.time_of_last_state_send = self.time_source.now();

            let message = UdpToClientMessage::<Game>::StateMessage(state_message);
            self.send_message(&message);

            //info!("state_message");
            self.log_time_in_queue(*time_in_queue);
        }

        return EventHandleResult::TryForNextEvent;
    }

    fn on_input_message(
        &mut self,
        receive_meta_data: ReceiveMetaData,
        input_message: InputMessage<Game>,
    ) -> EventHandleResult {
        let time_in_queue = receive_meta_data.get_send_meta_data().get_time_sent();

        if self.player_index != input_message.get_player_index()
            && (self.last_state_sequence.is_none()
                || self.last_state_sequence.as_ref().unwrap() <= &input_message.get_frame_index())
        {
            self.time_of_last_input_send = self.time_source.now();

            let message = UdpToClientMessage::<Game>::InputMessage(input_message);
            self.send_message(&message);

            //info!("input_message");
            self.log_time_in_queue(*time_in_queue);
        } else {
            //info!("InputMessage dropped. Last state: {:?}", tcp_output.last_state_sequence);
        }

        return EventHandleResult::TryForNextEvent;
    }

    fn on_server_input_message(
        &mut self,
        receive_meta_data: ReceiveMetaData,
        server_input_message: ServerInputMessage<Game>,
    ) -> EventHandleResult {
        let time_in_queue = receive_meta_data.get_send_meta_data().get_time_sent();

        if self.last_state_sequence.is_none()
            || self.last_state_sequence.as_ref().unwrap() <= &server_input_message.get_frame_index()
        {
            self.time_of_last_server_input_send = self.time_source.now();

            let message = UdpToClientMessage::<Game>::ServerInputMessage(server_input_message);
            self.send_message(&message);

            //info!("server_input_message");
            self.log_time_in_queue(*time_in_queue);
        } else {
            //info!("ServerInputMessage dropped. Last state: {:?}", tcp_output.last_state_sequence);
        }

        return EventHandleResult::TryForNextEvent;
    }

    fn send_ping_response(
        &mut self,
        time_received: TimeValue,
        ping_request: PingRequest,
    ) -> EventHandleResult {
        let ping_response = PingResponse::new(ping_request, time_received, self.time_source.now());
        let ping_response = UdpToClientMessage::PingResponse(ping_response);
        match self.send_message(&ping_response) {
            ControlFlow::Continue(()) => EventHandleResult::TryForNextEvent,
            ControlFlow::Break(()) => EventHandleResult::StopThread,
        }
    }

    //TODO: generalize this for all channels
    fn log_time_in_queue(&mut self, time_in_queue: TimeValue) {
        let now = self.time_source.now();
        let duration_in_queue = now.duration_since(&time_in_queue);

        self.time_in_queue_rolling_average
            .add_value(duration_in_queue.as_secs_f64());
        let average = self.time_in_queue_rolling_average.get_average();

        if average > 500.0 {
            warn!(
                "High average duration in queue: {:?} in milliseconds",
                average
            );
        }
    }

    fn send_message(&mut self, message: &UdpToClientMessage<Game>) -> ControlFlow<()> {
        let remote_peer = match &self.remote_peer {
            Some(remote_peer) => remote_peer,
            None => {
                warn!("Attempting to send without a peer address");
                return ControlFlow::Continue(());
            }
        };

        //TODO: see if this can be write
        let buf = rmp_serde::to_vec(&message).unwrap();
        let fragments = self.fragmenter.make_fragments(buf);

        for fragment in fragments {
            if fragment.get_whole_buf().len() > MAX_UDP_DATAGRAM_SIZE {
                error!(
                    "Datagram is larger than MAX_UDP_DATAGRAM_SIZE: {:?}",
                    fragment.get_whole_buf().len()
                );
            }

            if let Err(err) = self
                .socket
                .send_to(&fragment.get_whole_buf(), &remote_peer.get_socket_addr())
            {
                warn!("Error while sending: {:?}", err);
                return ControlFlow::Break(());
            }
        }

        return ControlFlow::Continue(());
    }
}

impl<Game: GameTrait> HandleEvent for UdpOutputEventHandler<Game> {
    type Event = UdpOutputEvent<Game>;
    type ThreadReturn = ();

    fn on_event(
        &mut self,
        receive_meta_data: ReceiveMetaData,
        event: Self::Event,
    ) -> EventHandleResult {
        let now = self.time_source.now();

        // let duration_since_last_input = now.duration_since(self.time_of_last_input_send);
        // if duration_since_last_input > TimeDuration::one_second() {
        //     warn!("It has been {:?} since last input message was sent. Now: {:?}, Last: {:?}, Queue length: {:?}",
        //           duration_since_last_input, now, self.time_of_last_input_send, self.input_queue.len());
        // }

        let duration_since_last_state = now.duration_since(&self.time_of_last_state_send);
        if duration_since_last_state > TimeDuration::ONE_SECOND {
            //TODO: this should probably be a warn when it happens less often
            debug!(
                "It has been {:?} since last state message was sent. Now: {:?}, Last: {:?}",
                duration_since_last_state, now, self.time_of_last_state_send
            );
        }

        match event {
            UdpOutputEvent::RemotePeer(remote_udp_peer) => self.on_remote_peer(remote_udp_peer),
            UdpOutputEvent::SendInputMessage(input_message) => {
                self.on_input_message(receive_meta_data, input_message)
            }
            UdpOutputEvent::SendServerInputMessage(server_input_message) => {
                self.on_server_input_message(receive_meta_data, server_input_message)
            }
            UdpOutputEvent::SendCompletedStep(state_message) => {
                self.on_completed_step(receive_meta_data, state_message)
            }
            UdpOutputEvent::PingRequest {
                time_received,
                ping_request,
            } => self.send_ping_response(time_received, ping_request),
        }
    }

    fn on_stop_self(self) -> Self::ThreadReturn {
        ()
    }
}
