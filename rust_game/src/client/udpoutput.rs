use log::{info, error, debug};
use crate::interface::GameTrait;
use std::net::{UdpSocket, SocketAddrV4};
use crate::messaging::{InputMessage, ToServerMessageUDP, InitialInformation, MAX_UDP_DATAGRAM_SIZE, Fragmenter};
use std::io;
use std::ops::ControlFlow::{Continue, Break};
use commons::threading::channel::ReceiveMetaData;
use commons::threading::eventhandling::{ChannelEvent, ChannelEventResult, EventHandlerTrait};
use commons::threading::eventhandling::WaitOrTryForNextEvent::{TryForNextEvent, WaitForNextEvent};

//TODO: combine server/client and tcp/udp inputs/outputs to shared listener/eventhandler types
pub enum UdpOutputEvent<Game: GameTrait> {
    InputMessageEvent(InputMessage<Game>)
}

pub struct UdpOutput<Game: GameTrait> {
    server_address: SocketAddrV4,
    socket: UdpSocket,
    fragmenter: Fragmenter,
    input_queue: Vec<InputMessage<Game>>,
    max_observed_input_queue: usize,
    initial_information: InitialInformation<Game>
}

impl<Game: GameTrait> UdpOutput<Game> {

    pub fn new(server_socket_addr_v4: SocketAddrV4,
               socket: UdpSocket,
               initial_information: InitialInformation<Game>) -> Self {

        let mut udp_output = Self {
            server_address: server_socket_addr_v4,
            socket,
            //TODO: make max datagram size more configurable
            fragmenter: Fragmenter::new(MAX_UDP_DATAGRAM_SIZE),
            input_queue: Vec::new(),
            max_observed_input_queue: 0,
            initial_information
        };

        let message = ToServerMessageUDP::<Game>::Hello{player_index: udp_output.initial_information.get_player_index()};
        udp_output.send_message(message);

        return udp_output;
    }

    pub fn on_input_message(&mut self, input_message: InputMessage<Game>) {
        //insert in reverse sorted order
        match self.input_queue.binary_search_by(|elem| { input_message.cmp(elem) }) {
            Ok(pos) => self.input_queue[pos] = input_message,
            Err(pos) => self.input_queue.insert(pos, input_message)
        }
    }

    fn send_all_messages(&mut self) {
        let mut send_another_message = true;
        while send_another_message {

            if self.input_queue.len() > self.max_observed_input_queue {
                self.max_observed_input_queue = self.input_queue.len();
                info!("Outbound input queue has hit a max size of {:?}", self.max_observed_input_queue);
            }

            match self.input_queue.pop() {
                None => send_another_message = false,
                Some(input_to_send) => {
                    let message = ToServerMessageUDP::<Game>::Input(input_to_send);
                    self.send_message(message);
                }
            }
        }
    }

    fn send_message(&mut self, message: ToServerMessageUDP<Game>) {

        let buf = rmp_serde::to_vec(&message).unwrap();
        let fragments = self.fragmenter.make_fragments(buf);

        for fragment in fragments {

            if fragment.get_whole_buf().len() > MAX_UDP_DATAGRAM_SIZE {
                error!("Datagram is larger than MAX_UDP_DATAGRAM_SIZE: {:?}", fragment.get_whole_buf().len());
            }

            self.socket.send_to(fragment.get_whole_buf(), &self.server_address).unwrap();
        }
    }
}

impl<Game: GameTrait> EventHandlerTrait for UdpOutput<Game> {
    type Event = UdpOutputEvent<Game>;
    type ThreadReturn = ();

    fn on_channel_event(mut self, channel_event: ChannelEvent<Self::Event>) -> ChannelEventResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(_, event) => {
                match event {
                    UdpOutputEvent::InputMessageEvent(input_message) => self.on_input_message(input_message)
                };

                return Continue(TryForNextEvent(self));
            }
            ChannelEvent::Timeout => {
                self.send_all_messages();
                return Continue(WaitForNextEvent(self));
            },
            ChannelEvent::ChannelEmpty => {
                self.send_all_messages();
                return Continue(WaitForNextEvent(self));
            }
            ChannelEvent::ChannelDisconnected => {
                return Break(());
            }
        }
    }

    fn on_stop(self, _: ReceiveMetaData) -> Self::ThreadReturn { () }
}