use {
    crate::{
        FrameIndex,
        Play,
        game_time::PingRequest,
        interface::InitialInformation,
        messaging::{
            Fragmenter,
            ToServerInputMessage,
            UdpToServerMessage,
        },
    },
    commons::{
        real_time::{
            EventHandleResult,
            EventHandlerBuilder,
            EventSender,
            Factory,
            HandleEvent,
            ReceiveMetaData,
            TimeSource,
            net::{
                MAX_UDP_DATAGRAM_SIZE,
                udp::UdpSocket,
            },
        },
        utils::unit_error,
    },
    log::error,
    std::{
        io::Error,
        net::SocketAddr,
    },
};

enum Event<Game: Play> {
    InputMessageEvent(ToServerInputMessage<Game>),
    FrameIndex(FrameIndex),
}

#[derive(Clone)]
pub struct UdpOutput<Game: Play> {
    sender: EventSender<Event<Game>>,
}

impl<Game: Play> UdpOutput<Game> {
    pub fn new(
        factory: &Factory,
        server_address: SocketAddr,
        socket: UdpSocket,
        initial_information: InitialInformation<Game>,
    ) -> Result<Self, Error> {
        let ping_period_frames = initial_information
            .get_server_config()
            .get_frame_duration()
            .to_frame_count(&Game::PING_PERIOD) as usize;

        let event_handler = EventHandler {
            time_source: factory.get_time_source().clone(),
            server_address,
            socket,
            ping_period_frames,
            next_ping: FrameIndex::zero(),
            //TODO: make max datagram size more configurable
            fragmenter: Fragmenter::new(MAX_UDP_DATAGRAM_SIZE),
            initial_information,
            non_authoritative_input: None,
        };

        //TODO: unwrap after try_clone is not good
        let sender =
            EventHandlerBuilder::new_thread(factory, "ClientUdpOutput".to_string(), event_handler)?;

        Ok(Self { sender })
    }

    pub fn advance_frame_index(&self, frame_index: FrameIndex) -> Result<(), ()> {
        let event = Event::FrameIndex(frame_index);
        self.sender.send_event(event).map_err(unit_error)
    }

    //TODO: add method/logic for non-authoritative inputs
    pub fn send_input(&self, input: ToServerInputMessage<Game>) -> Result<(), ()> {
        let event = Event::InputMessageEvent(input);
        self.sender.send_event(event).map_err(unit_error)
    }

    pub fn stop(&self) -> Result<(), ()> {
        self.sender.send_stop_thread()
    }
}

struct EventHandler<Game: Play> {
    time_source: TimeSource,
    server_address: SocketAddr,
    socket: UdpSocket,
    ping_period_frames: usize,
    next_ping: FrameIndex,
    fragmenter: Fragmenter,
    initial_information: InitialInformation<Game>,
    non_authoritative_input: Option<ToServerInputMessage<Game>>,
}

impl<Game: Play> EventHandler<Game> {
    fn on_input_message(&mut self, input_message: ToServerInputMessage<Game>) -> EventHandleResult {
        if input_message.get_input().is_authoritative() {
            // If the authoritative input is as new or newer than the pending
            // non-authoritative one, don't bother to send the non-authoritative
            // one
            if let Some(non_authoritative_message) = &self.non_authoritative_input {
                if input_message.get_input_coordinates().get_frame_index()
                    >= non_authoritative_message
                        .get_input_coordinates()
                        .get_frame_index()
                {
                    self.non_authoritative_input = None;
                }
            }

            let message = UdpToServerMessage::<Game>::Input(input_message);
            if self.send_message(&message).is_err() {
                return EventHandleResult::StopThread;
            }
        } else {
            match &self.non_authoritative_input {
                Some(pending)
                    if pending.get_input_coordinates().get_frame_index()
                        <= input_message.get_input_coordinates().get_frame_index() =>
                {
                    self.non_authoritative_input = Some(input_message)
                }
                None => self.non_authoritative_input = Some(input_message),
                _ => {
                    #[cfg(debug_assertions)]
                    panic!("An attempt was made to send an outdated non-authoritative input");
                }
            }
        }

        return EventHandleResult::TryForNextEvent;
    }

    fn send_message(&mut self, message: &UdpToServerMessage<Game>) -> Result<(), ()> {

        /*
        TODO:

use something like a fillable buff

struct FallibleWriter(Vec<u8>);

impl Write for FallibleWriter {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write_all(buf)?;
        Ok(buf.len())
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.0.try_reserve(buf.len()).map_err(|_| std::io::ErrorKind::OutOfMemory)?;
        self.0.extend_from_slice(buf);
        Ok(())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
        
         */

        //TODO: use write instead of to_vec
        let buf = rmp_serde::to_vec(&message).unwrap();
        let fragments = self.fragmenter.make_fragments(&buf);

        for fragment in fragments {
            let result = self
                .socket
                .send_to(fragment.get_whole_buf(), &self.server_address);

            if let Err(err) = result {
                error!("Error sending a UDP message.\n{:?}", err);
                return Err(());
            }
        }

        return Ok(());
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
        if self.send_message(&ping_request).is_err() {
            return EventHandleResult::StopThread;
        }

        return EventHandleResult::TryForNextEvent;
    }
}

impl<Game: Play> HandleEvent for EventHandler<Game> {
    type Event = Event<Game>;
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, event: Self::Event) -> EventHandleResult {
        match event {
            Event::InputMessageEvent(input_message) => self.on_input_message(input_message),
            Event::FrameIndex(frame_index) => self.on_frame_index(frame_index),
        }
    }
    fn on_channel_empty(&mut self) -> EventHandleResult {
        if let Some(input_message) = self.non_authoritative_input.take() {
            let message = UdpToServerMessage::<Game>::Input(input_message);
            if self.send_message(&message).is_err() {
                return EventHandleResult::StopThread;
            };
        }

        return EventHandleResult::WaitForNextEvent;
    }

    fn on_stop_self(self) -> Self::ThreadReturn {
        ()
    }
}
