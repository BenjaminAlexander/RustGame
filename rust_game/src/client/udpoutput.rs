use log::{info, error, debug};
use crate::interface::GameTrait;
use std::net::{UdpSocket, SocketAddrV4};
use crate::messaging::{InputMessage, ToServerMessageUDP, InitialInformation, MAX_UDP_DATAGRAM_SIZE, Fragmenter};
use std::io;
use std::sync::mpsc::TryRecvError;
use crate::threading::{ChannelThread, Receiver, ChannelDrivenThreadSender as Sender, ThreadAction};

pub struct UdpOutput<Game: GameTrait> {
    server_address: SocketAddrV4,
    socket: UdpSocket,
    fragmenter: Fragmenter,
    input_queue: Vec<InputMessage<Game>>,
    max_observed_input_queue: usize,
    initial_information: Option<InitialInformation<Game>>
}

impl<Game: GameTrait> UdpOutput<Game> {

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

impl<Game: GameTrait> ChannelThread<(), ThreadAction> for UdpOutput<Game> {

    fn run(mut self, receiver: Receiver<Self, ThreadAction>) -> () {

        loop {
            loop {
                match receiver.try_recv(&mut self) {
                    Ok(ThreadAction::Continue) => {}
                    Err(TryRecvError::Empty) => break,
                    Ok(ThreadAction::Stop) => {
                        info!("Thread commanded to stop.");
                        return;
                    }
                    Err(TryRecvError::Disconnected) => {
                        info!("Thread stopped due to disconnect");
                        return;
                    }
                }
            }

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
    }
}

impl<Game: GameTrait> Sender<UdpOutput<Game>> {

    pub fn on_initial_information(&self, initial_information: InitialInformation<Game>) {
        self.send(move |udp_output|{
            debug!("InitialInformation Received.");
            udp_output.initial_information = Some(initial_information);

            let message = ToServerMessageUDP::<Game>::Hello{player_index: udp_output.initial_information.as_ref().unwrap().get_player_index()};
            udp_output.send_message(message);

            return ThreadAction::Continue;
        }).unwrap();
    }

    pub fn on_input_message(&self, input_message: InputMessage<Game>) {
        self.send(move |udp_output|{

            //insert in reverse sorted order
            match udp_output.input_queue.binary_search_by(|elem| { input_message.cmp(elem) }) {
                Ok(pos) => udp_output.input_queue[pos] = input_message,
                Err(pos) => udp_output.input_queue.insert(pos, input_message)
            }

            return ThreadAction::Continue;
        }).unwrap();
    }
}