use log::{trace, info, error};
use crate::interface::{Game, Input, State};
use std::net::{UdpSocket, Ipv4Addr, SocketAddrV4};
use crate::messaging::{InputMessage, ToServerMessageUDP, InitialInformation, MAX_UDP_DATAGRAM_SIZE, Fragmenter};
use std::io;
use crate::threading::{ChannelThread, Receiver, Consumer, Sender};
use std::str::FromStr;

pub struct UdpOutput<GameType: Game> {
    server_address: SocketAddrV4,
    socket: UdpSocket,
    fragmenter: Fragmenter,
    input_queue: Vec<InputMessage<GameType>>,
    max_observed_input_queue: usize,
    initial_information: Option<InitialInformation<GameType>>
}

impl<GameType: Game> UdpOutput<GameType> {

    pub fn new(server_socket_addr_v4: SocketAddrV4,
               socket: &UdpSocket) -> io::Result<Self> {

        return Ok(Self{
            server_address: server_socket_addr_v4,
            socket: socket.try_clone()?,
            //TODO: make max datagram size more configurable
            fragmenter: Fragmenter::new(MAX_UDP_DATAGRAM_SIZE),
            input_queue: Vec::new(),
            max_observed_input_queue: 0,
            initial_information: None
        });
    }

    fn send_message(&mut self, message: ToServerMessageUDP<GameType>) {

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

impl<GameType: Game> ChannelThread<()> for UdpOutput<GameType> {

    fn run(mut self, receiver: Receiver<Self>) -> () {

        loop {
            trace!("Waiting.");
            match receiver.recv(&mut self) {
                Err(_error) => {
                    info!("Channel closed.");
                    return ();
                }
                _ => {}
            }

            let mut send_another_message = true;
            while send_another_message {
                receiver.try_iter(&mut self);

                if self.input_queue.len() > self.max_observed_input_queue {
                    self.max_observed_input_queue = self.input_queue.len();
                    info!("Outbound input queue has hit a max size of {:?}", self.max_observed_input_queue);
                }

                match self.input_queue.pop() {
                    None => send_another_message = false,
                    Some(input_to_send) => {
                        let message = ToServerMessageUDP::<GameType>::Input(input_to_send);
                        self.send_message(message);
                    }
                }
            }
        }
    }
}

impl<GameType: Game> Consumer<InitialInformation<GameType>> for Sender<UdpOutput<GameType>> {

    fn accept(&self, initial_information: InitialInformation<GameType>) {
        self.send(move |udp_output|{
            info!("InitialInformation Received.");
            udp_output.initial_information = Some(initial_information);

            let message = ToServerMessageUDP::<GameType>::Hello{player_index: udp_output.initial_information.as_ref().unwrap().get_player_index()};
            udp_output.send_message(message);

        }).unwrap();
    }
}

impl<GameType: Game> Consumer<InputMessage<GameType>> for Sender<UdpOutput<GameType>> {

    fn accept(&self, input_message: InputMessage<GameType>) {
        self.send(move |udp_output|{

            //insert in reverse sorted order
            match udp_output.input_queue.binary_search_by(|elem| { input_message.cmp(elem) }) {
                Ok(pos) => udp_output.input_queue[pos] = input_message,
                Err(pos) => udp_output.input_queue.insert(pos, input_message)
            }

        }).unwrap();
    }
}