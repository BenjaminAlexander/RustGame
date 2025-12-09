use crate::game_time::PingRequest;
use crate::interface::{
    GameTrait,
    InitialInformation,
};
use crate::messaging::{
    Fragmenter,
    ToServerInputMessage,
    UdpToServerMessage,
};
use crate::FrameIndex;
use commons::real_time::net::udp::UdpSocket;
use commons::real_time::net::MAX_UDP_DATAGRAM_SIZE;
use commons::real_time::{
    EventHandleResult,
    HandleEvent,
    ReceiveMetaData,
    TimeSource,
};
use log::error;
use std::net::SocketAddr;

//TODO: combine server/client and tcp/udp inputs/outputs to shared listener/eventhandler types
pub enum UdpOutputEvent<Game: GameTrait> {
    InputMessageEvent(ToServerInputMessage<Game>),
    FrameIndex(FrameIndex),
}

pub struct UdpOutput<Game: GameTrait> {
    time_source: TimeSource,
    server_address: SocketAddr,
    socket: UdpSocket,
    ping_period_frames: usize,
    next_ping: FrameIndex,
    fragmenter: Fragmenter,
    initial_information: InitialInformation<Game>,
}

impl<Game: GameTrait> UdpOutput<Game> {
    pub fn new(
        time_source: TimeSource,
        server_address: SocketAddr,
        socket: UdpSocket,
        initial_information: InitialInformation<Game>,
    ) -> Self {
        let ping_period_frames = initial_information
            .get_server_config()
            .get_frame_duration()
            .to_frame_count(&Game::PING_PERIOD) as usize;

        Self {
            time_source,
            server_address,
            socket,
            ping_period_frames,
            next_ping: FrameIndex::zero(),
            //TODO: make max datagram size more configurable
            fragmenter: Fragmenter::new(MAX_UDP_DATAGRAM_SIZE),
            initial_information,
        }
    }

    fn on_input_message(&mut self, input_message: ToServerInputMessage<Game>) -> EventHandleResult {
        let message = UdpToServerMessage::<Game>::Input(input_message);
        self.send_message(&message);

        return EventHandleResult::TryForNextEvent;
    }

    fn send_message(&mut self, message: &UdpToServerMessage<Game>) {
        //TODO: use write instead of to_vec
        let buf = rmp_serde::to_vec(&message).unwrap();
        let fragments = self.fragmenter.make_fragments(buf);

        for fragment in fragments {
            if fragment.get_whole_buf().len() > MAX_UDP_DATAGRAM_SIZE {
                error!(
                    "Datagram is larger than MAX_UDP_DATAGRAM_SIZE: {:?}",
                    fragment.get_whole_buf().len()
                );
            }

            self.socket
                .send_to(fragment.get_whole_buf(), &self.server_address)
                .unwrap();
        }
    }

    fn on_frame_index(&mut self, frame_index: FrameIndex) -> EventHandleResult {
        if frame_index < self.next_ping {
            return EventHandleResult::TryForNextEvent;
        }

        self.next_ping = frame_index + self.ping_period_frames;

        let ping_request = PingRequest::new(
            self.initial_information.get_player_index(),
            self.time_source.now(),
        );
        let ping_request = UdpToServerMessage::PingRequest(ping_request);
        self.send_message(&ping_request);
        return EventHandleResult::TryForNextEvent;
    }
}

impl<Game: GameTrait> HandleEvent for UdpOutput<Game> {
    type Event = UdpOutputEvent<Game>;
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, event: Self::Event) -> EventHandleResult {
        match event {
            UdpOutputEvent::InputMessageEvent(input_message) => {
                self.on_input_message(input_message)
            }
            UdpOutputEvent::FrameIndex(frame_index) => self.on_frame_index(frame_index),
        }
    }

    fn on_stop_self(self) -> Self::ThreadReturn {
        ()
    }
}
