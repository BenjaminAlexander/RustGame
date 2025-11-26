use crate::game_time::PingRequest;
use crate::interface::{
    GameTrait,
    InitialInformation,
};
use crate::messaging::{
    FragmentableUdpToServerMessage,
    Fragmenter,
    InputMessage,
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
use log::{
    error,
    info,
};
use std::io::Error;
use std::net::SocketAddr;

//TODO: combine server/client and tcp/udp inputs/outputs to shared listener/eventhandler types
pub enum UdpOutputEvent<Game: GameTrait> {
    InputMessageEvent(InputMessage<Game>),
    FrameIndex(FrameIndex),
}

pub struct UdpOutput<Game: GameTrait> {
    time_source: TimeSource,
    server_address: SocketAddr,
    socket: UdpSocket,
    ping_period_frames: usize,
    next_ping: FrameIndex,
    fragmenter: Fragmenter,
    input_queue: Vec<InputMessage<Game>>,
    max_observed_input_queue: usize,
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
            .to_frame_count(&Game::TIME_SYNC_MESSAGE_PERIOD)
            as usize;

        let mut udp_output = Self {
            time_source,
            server_address,
            socket,
            ping_period_frames,
            next_ping: FrameIndex::zero(),
            //TODO: make max datagram size more configurable
            fragmenter: Fragmenter::new(MAX_UDP_DATAGRAM_SIZE),
            input_queue: Vec::new(),
            max_observed_input_queue: 0,
            initial_information,
        };

        let message = FragmentableUdpToServerMessage::<Game>::Hello {
            player_index: udp_output.initial_information.get_player_index(),
        };
        udp_output.send_fragmentable_message(message);

        return udp_output;
    }

    fn on_input_message(&mut self, input_message: InputMessage<Game>) -> EventHandleResult {
        //insert in reverse sorted order
        match self
            .input_queue
            .binary_search_by(|elem| input_message.cmp(elem))
        {
            Ok(pos) => self.input_queue[pos] = input_message,
            Err(pos) => self.input_queue.insert(pos, input_message),
        };

        return EventHandleResult::TryForNextEvent;
    }

    fn send_all_messages(&mut self) {
        let mut send_another_message = true;
        while send_another_message {
            if self.input_queue.len() > self.max_observed_input_queue {
                self.max_observed_input_queue = self.input_queue.len();
                info!(
                    "Outbound input queue has hit a max size of {:?}",
                    self.max_observed_input_queue
                );
            }

            match self.input_queue.pop() {
                None => send_another_message = false,
                Some(input_to_send) => {
                    let message = FragmentableUdpToServerMessage::<Game>::Input(input_to_send);
                    self.send_fragmentable_message(message);
                }
            }
        }
    }

    fn send_fragmentable_message(&mut self, message: FragmentableUdpToServerMessage<Game>) {
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

            let fragment = UdpToServerMessage::Fragment(fragment.move_whole_buf());
            self.send_message(&fragment).unwrap();
        }
    }

    fn send_message(&mut self, message: &UdpToServerMessage) -> Result<usize, Error> {
        //TODO: use write instead of to_vec
        let buf = rmp_serde::to_vec(&message).unwrap();

        self.socket.send_to(&buf, &self.server_address)
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
        match self.send_message(&ping_request) {
            Ok(_) => EventHandleResult::TryForNextEvent,
            Err(err) => {
                error!("Failed to send ping request: {:?}", err);
                EventHandleResult::StopThread
            }
        }
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

    fn on_channel_empty(&mut self) -> EventHandleResult {
        self.send_all_messages();
        return EventHandleResult::WaitForNextEvent;
    }

    fn on_timeout(&mut self) -> EventHandleResult {
        self.send_all_messages();
        return EventHandleResult::WaitForNextEvent;
    }

    fn on_stop_self(self) -> Self::ThreadReturn {
        ()
    }
}
