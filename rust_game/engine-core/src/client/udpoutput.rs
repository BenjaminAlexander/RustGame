use log::{info, error};
use crate::interface::{GameFactoryTrait, GameTrait, UdpSocket};
use std::net::SocketAddr;
use crate::messaging::{InputMessage, ToServerMessageUDP, InitialInformation, Fragmenter};
use std::ops::ControlFlow::{Continue, Break};
use commons::net::{MAX_UDP_DATAGRAM_SIZE, UdpSocketTrait};
use commons::threading::channel::ReceiveMetaData;
use commons::threading::eventhandling::{ChannelEvent, EventHandleResult, EventHandlerTrait};
use commons::threading::eventhandling::WaitOrTryForNextEvent::{TryForNextEvent, WaitForNextEvent};

//TODO: combine server/client and tcp/udp inputs/outputs to shared listener/eventhandler types
pub enum UdpOutputEvent<Game: GameTrait> {
    InputMessageEvent(InputMessage<Game>)
}

pub struct UdpOutput<GameFactory: GameFactoryTrait> {
    server_address: SocketAddr,
    socket: UdpSocket<GameFactory>,
    fragmenter: Fragmenter,
    input_queue: Vec<InputMessage<GameFactory::Game>>,
    max_observed_input_queue: usize,
    initial_information: InitialInformation<GameFactory::Game>
}

impl<GameFactory: GameFactoryTrait> UdpOutput<GameFactory> {

    pub fn new(server_address: SocketAddr,
               socket: UdpSocket<GameFactory>,
               initial_information: InitialInformation<GameFactory::Game>) -> Self {

        let mut udp_output = Self {
            server_address,
            socket,
            //TODO: make max datagram size more configurable
            fragmenter: Fragmenter::new(MAX_UDP_DATAGRAM_SIZE),
            input_queue: Vec::new(),
            max_observed_input_queue: 0,
            initial_information
        };

        let message = ToServerMessageUDP::<GameFactory::Game>::Hello{player_index: udp_output.initial_information.get_player_index()};
        udp_output.send_message(message);

        return udp_output;
    }

    pub fn on_input_message(&mut self, input_message: InputMessage<GameFactory::Game>) {
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
                    let message = ToServerMessageUDP::<GameFactory::Game>::Input(input_to_send);
                    self.send_message(message);
                }
            }
        }
    }

    fn send_message(&mut self, message: ToServerMessageUDP<GameFactory::Game>) {

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

impl<GameFactory: GameFactoryTrait> EventHandlerTrait for UdpOutput<GameFactory> {
    type Event = UdpOutputEvent<GameFactory::Game>;
    type ThreadReturn = ();

    fn on_channel_event(mut self, channel_event: ChannelEvent<Self::Event>) -> EventHandleResult<Self> {
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