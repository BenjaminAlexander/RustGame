use log::{trace, info};
use crate::interface::{Input, State};
use std::net::{UdpSocket, Ipv4Addr, SocketAddrV4};
use crate::messaging::{InputMessage, ToServerMessageUDP, InitialInformation};
use std::io;
use crate::threading::{ChannelThread, Receiver, Consumer, Sender};
use std::str::FromStr;

pub struct UdpOutput<StateType: State<InputType>, InputType: Input> {
    server_address: SocketAddrV4,
    socket: UdpSocket,
    input_queue: Vec<InputMessage<InputType>>,
    initial_information: Option<InitialInformation<StateType>>
}

impl<StateType: State<InputType>, InputType: Input> UdpOutput<StateType, InputType> {

    pub fn new(server_ip: String,
               dst_port: u16,
               socket: &UdpSocket) -> io::Result<Self> {

        let addr_v4 = Ipv4Addr::from_str(server_ip.as_str()).unwrap();
        let socket_addr_v4 = SocketAddrV4::new(addr_v4, dst_port);
        //let socket_addr:SocketAddr = SocketAddr::from(socket_addr_v4);
        //let tcp_stream = TcpStream::connect(socket_addr).unwrap();

        return Ok(Self{
            server_address: socket_addr_v4,
            socket: socket.try_clone()?,
            input_queue: Vec::new(),
            initial_information: None
        });
    }
}

impl<StateType: State<InputType>, InputType: Input> ChannelThread<()> for UdpOutput<StateType, InputType> {

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

                match self.input_queue.pop() {
                    None => send_another_message = false,
                    Some(input_to_send) => {
                        let message = ToServerMessageUDP::<InputType>::Input(input_to_send);
                        let buf = rmp_serde::to_vec(&message).unwrap();
                        self.socket.send_to(&buf, &self.server_address).unwrap();
                    }
                }
            }
        }
    }
}

impl<StateType, InputType> Consumer<InitialInformation<StateType>> for Sender<UdpOutput<StateType, InputType>>
    where StateType: State<InputType>,
          InputType: Input {

    fn accept(&self, initial_information: InitialInformation<StateType>) {
        self.send(move |udp_output|{
            info!("InitialInformation Received.");
            udp_output.initial_information = Some(initial_information);

            let message = ToServerMessageUDP::<InputType>::Hello{player_index: udp_output.initial_information.as_ref().unwrap().get_player_index()};
            let buf = rmp_serde::to_vec(&message).unwrap();
            udp_output.socket.send_to(&buf, &udp_output.server_address).unwrap();

        }).unwrap();
    }
}

impl<StateType: State<InputType>, InputType: Input> Consumer<InputMessage<InputType>> for Sender<UdpOutput<StateType, InputType>> {

    fn accept(&self, input_message: InputMessage<InputType>) {
        self.send(move |udp_output|{

            //insert in reverse sorted order
            match udp_output.input_queue.binary_search_by(|elem| { input_message.cmp(elem) }) {
                Ok(pos) => udp_output.input_queue[pos] = input_message,
                Err(pos) => udp_output.input_queue.insert(pos, input_message)
            }

        }).unwrap();
    }
}