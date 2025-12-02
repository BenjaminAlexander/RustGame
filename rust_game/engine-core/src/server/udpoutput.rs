use crate::game_time::{
    PingRequest,
    PingResponse,
};
use crate::interface::GameTrait;
use crate::messaging::{
    Fragmenter, StateMessage, ToClientInputMessage, UdpToClientMessage
};
use crate::server::remoteudppeer::RemoteUdpPeer;
use commons::real_time::net::udp::UdpSocket;
use commons::real_time::net::MAX_UDP_DATAGRAM_SIZE;
use commons::real_time::{
    EventHandleResult,
    EventHandlerBuilder,
    EventSender,
    Factory,
    HandleEvent,
    ReceiveMetaData,
    TimeSource,
};
use commons::time::TimeValue;
use commons::utils::unit_error;
use log::{
    error,
    info,
    warn,
};
use std::io::Error;
use std::marker::PhantomData;
use std::ops::ControlFlow;

#[derive(Clone)]
pub struct UdpOutput<Game: GameTrait> {
    sender: EventSender<Event<Game>>,
}

impl<Game: GameTrait> UdpOutput<Game> {
    pub fn new(
        factory: Factory,
        player_index: usize,
        udp_socket: &UdpSocket,
    ) -> Result<Self, Error> {
        let event_handler = EventHandler::<Game>::new(
            factory.get_time_source().clone(),
            player_index,
            &udp_socket,
        )?;

        let sender = EventHandlerBuilder::new_thread(
            &factory,
            format!("ServerUdpOutput-Player-{}", player_index),
            event_handler,
        )?;

        Ok(Self { sender })
    }

    pub fn send_ping_response(
        &self,
        time_received: TimeValue,
        ping_request: PingRequest,
    ) -> Result<(), ()> {
        let event = Event::PingRequest {
            time_received,
            ping_request,
        };

        self.sender.send_event(event).map_err(unit_error)
    }

    pub fn set_remote_peer(&self, remote_udp_peer: RemoteUdpPeer) -> Result<(), ()> {
        let event = Event::RemotePeer(remote_udp_peer);
        self.sender.send_event(event).map_err(unit_error)
    }

    pub fn send_input_message(&self, input_message: ToClientInputMessage<Game>) -> Result<(), ()> {
        let event = Event::SendInputMessage(input_message);
        self.sender.send_event(event).map_err(unit_error)
    }

    pub fn send_completed_step(&self, step_message: StateMessage<Game>) -> Result<(), ()> {
        let event = Event::SendCompletedStep(step_message);
        self.sender.send_event(event).map_err(unit_error)
    }
}

enum Event<Game: GameTrait> {
    RemotePeer(RemoteUdpPeer),
    PingRequest {
        time_received: TimeValue,
        ping_request: PingRequest,
    },
    SendInputMessage(ToClientInputMessage<Game>),
    SendCompletedStep(StateMessage<Game>),
}

struct EventHandler<Game: GameTrait> {
    time_source: TimeSource,
    player_index: usize,
    socket: UdpSocket,
    remote_peer: Option<RemoteUdpPeer>,
    fragmenter: Fragmenter,
    phantom: PhantomData<Game>,
}

impl<Game: GameTrait> EventHandler<Game> {
    pub fn new(
        time_source: TimeSource,
        player_index: usize,
        socket: &UdpSocket,
    ) -> Result<Self, Error> {
        Ok(EventHandler {
            player_index,
            remote_peer: None,
            //TODO: move clone outside
            socket: socket.try_clone()?,
            //TODO: make max datagram size more configurable
            fragmenter: Fragmenter::new(MAX_UDP_DATAGRAM_SIZE),
            phantom: PhantomData,
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
        state_message: StateMessage<Game>,
    ) -> EventHandleResult {
        let message = UdpToClientMessage::<Game>::StateMessage(state_message);
        self.send_message(&message);
        return EventHandleResult::TryForNextEvent;
    }

    fn on_input_message(
        &mut self,
        input_message: ToClientInputMessage<Game>,
    ) -> EventHandleResult {
        let message = UdpToClientMessage::<Game>::InputMessage(input_message);
        self.send_message(&message);

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

impl<Game: GameTrait> HandleEvent for EventHandler<Game> {
    type Event = Event<Game>;
    type ThreadReturn = ();

    fn on_event(
        &mut self,
        _receive_meta_data: ReceiveMetaData,
        event: Self::Event,
    ) -> EventHandleResult {
        match event {
            Event::RemotePeer(remote_udp_peer) => self.on_remote_peer(remote_udp_peer),
            Event::SendInputMessage(input_message) => {
                self.on_input_message(input_message)
            }
            Event::SendCompletedStep(state_message) => {
                self.on_completed_step(state_message)
            }
            Event::PingRequest {
                time_received,
                ping_request,
            } => self.send_ping_response(time_received, ping_request),
        }
    }

    fn on_stop_self(self) -> Self::ThreadReturn {
        ()
    }
}
