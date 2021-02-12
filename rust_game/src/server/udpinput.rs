use log::{error, info, trace};
use crate::messaging::{InputMessage, MAX_UDP_DATAGRAM_SIZE, ToServerMessageUDP};
use crate::threading::{ConsumerList, ChannelThread, Receiver, Sender, Consumer};
use crate::interface::Input;
use crate::server::tcpinput::TcpInput;
use std::net::{UdpSocket, SocketAddr};
use std::io;
use rmp_serde::decode::Error;
use crate::threading::sender::SendError;
use crate::server::remoteudppeer::RemoteUdpPeer;

pub struct UdpInput<InputType: Input> {
    socket: UdpSocket,
    remote_peers: Vec<Option<RemoteUdpPeer>>,
    input_consumers: ConsumerList<InputMessage<InputType>>,
    remote_peer_consumers: ConsumerList<RemoteUdpPeer>
}

impl<InputType: Input> UdpInput<InputType> {

    pub fn new(socket: &UdpSocket) -> io::Result<Self> {
        return Ok(Self{
            socket: socket.try_clone()?,
            remote_peers: Vec::new(),
            input_consumers: ConsumerList::new(),
            remote_peer_consumers: ConsumerList::new(),
        });
    }

    fn handle_remote_peer(&mut self, player_index: usize, remote_peer: SocketAddr) {
        while self.remote_peers.len() <= player_index {
            self.remote_peers.push(None);
        }

        let remote_peer = RemoteUdpPeer::new(player_index, remote_peer);

        //TODO: check peer against TCP address
        //TODO: Drop message if its from a bad address
        match &self.remote_peers[player_index] {
            None => {
                info!("First time UDP remote peer: {:?}", remote_peer);
                self.remote_peers[player_index] = Some(remote_peer);
                self.remote_peer_consumers.accept(self.remote_peers[player_index].as_ref().unwrap());
            }
            Some(existing_remote_peer) => {
                if !existing_remote_peer.eq(&remote_peer) {
                    info!("Change of UDP remote peer: {:?}", remote_peer);
                    self.remote_peers[player_index] = Some(remote_peer);
                    self.remote_peer_consumers.accept(self.remote_peers[player_index].as_ref().unwrap());
                }
            }
        }
    }
}

impl<InputType: Input> ChannelThread<()> for UdpInput<InputType> {

    fn run(mut self, receiver: Receiver<Self>) -> () {

        info!("Starting.");

        loop {

            let mut buf = [0; MAX_UDP_DATAGRAM_SIZE];
            let (number_of_bytes, source) = self.socket.recv_from(&mut buf).unwrap();
            let filled_buf = &mut buf[..number_of_bytes];

            let result: Result<ToServerMessageUDP::<InputType>, Error> = rmp_serde::from_read_ref(filled_buf);

            match result {
                Ok(message) => {

                    receiver.try_iter(&mut self);

                    self.handle_remote_peer(message.get_player_index(), source);

                    match message {
                        ToServerMessageUDP::Hello { player_index } => {

                        }
                        ToServerMessageUDP::Input(input_message) => {
                            self.input_consumers.accept(&input_message);
                        }
                    }
                }
                Err(error) => {
                    //TODO: tolerate bad packets
                    error!("Ending due to: {:?}", error);
                    return;
                }
            }
        }
    }
}

impl<InputType: Input> Sender<UdpInput<InputType>> {

    pub fn add_input_consumer<T>(&self, consumer: T) -> Result<(), SendError<UdpInput<InputType>>>
        where T: Consumer<InputMessage<InputType>> {

        self.send(|udp_input|{
            udp_input.input_consumers.add_consumer(consumer);
        })
    }

    pub fn add_remote_peer_consumers<T>(&self, consumer: T) -> Result<(), SendError<UdpInput<InputType>>>
        where T: Consumer<RemoteUdpPeer> {

        self.send(|udp_input|{

            for option in &udp_input.remote_peers {
                if let Some(remote_peer) = option {
                    consumer.accept(remote_peer.clone());
                }
            }

            udp_input.remote_peer_consumers.add_consumer(consumer);
        })
    }
}