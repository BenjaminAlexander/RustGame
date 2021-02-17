use log::{error, info, trace, warn};
use crate::messaging::{InputMessage, MAX_UDP_DATAGRAM_SIZE, ToServerMessageUDP, FragmentAssembler, MessageFragment};
use crate::threading::{ConsumerList, ChannelThread, Receiver, Sender, Consumer};
use crate::interface::Input;
use crate::server::tcpinput::TcpInput;
use std::net::{UdpSocket, SocketAddr, IpAddr};
use std::io;
use rmp_serde::decode::Error;
use crate::threading::sender::SendError;
use crate::server::remoteudppeer::RemoteUdpPeer;
use std::collections::{HashMap, HashSet};
use crate::server::clientaddress::ClientAddress;

pub struct UdpInput<InputType: Input> {
    socket: UdpSocket,
    remote_peers: Vec<Option<RemoteUdpPeer>>,
    client_addresses: Vec<Option<ClientAddress>>,
    client_ip_set: HashSet<IpAddr>,
    fragment_assemblers: HashMap<SocketAddr, FragmentAssembler>,
    input_consumers: ConsumerList<InputMessage<InputType>>,
    remote_peer_consumers: ConsumerList<RemoteUdpPeer>
}

impl<InputType: Input> UdpInput<InputType> {

    pub fn new(socket: &UdpSocket) -> io::Result<Self> {
        return Ok(Self{
            socket: socket.try_clone()?,
            remote_peers: Vec::new(),
            client_addresses: Vec::new(),
            client_ip_set: HashSet::new(),
            //TODO: make this more configurable
            fragment_assemblers: HashMap::new(),
            input_consumers: ConsumerList::new(),
            remote_peer_consumers: ConsumerList::new(),
        });
    }

    fn handle_receive(&mut self, buf: &mut [u8], source: SocketAddr) {
        if !self.client_ip_set.contains(&source.ip()) {
            warn!("Unexpected UDP packet received from {:?}", source);
            return;
        }

        if let Some(assembled) = self.handle_fragment(source, buf) {
            let result: Result<ToServerMessageUDP::<InputType>, Error> = rmp_serde::from_read_ref(assembled.as_slice());

            match result {
                Ok(message) => {
                    self.handle_message(message, source);
                }
                Err(error) => {
                    self.fragment_assemblers.remove(&source);
                    warn!("Failed to deserialize a ToServerMessageUDP: {:?}", error);
                    return;
                }
            }
        }
    }

    fn handle_fragment(&mut self, source: SocketAddr, fragment: &mut [u8]) -> Option<Vec<u8>> {
        let assembler = match self.fragment_assemblers.get_mut(&source) {
            None => {
                //TODO: make max_messages more configurable
                self.fragment_assemblers.insert(source, FragmentAssembler::new(5));
                self.fragment_assemblers.get_mut(&source).unwrap()
            }
            Some(assembler) => assembler
        };

        return assembler.add_fragment(MessageFragment::from_vec(fragment.to_vec()));
    }

    fn handle_message(&mut self, message: ToServerMessageUDP::<InputType>, source: SocketAddr) {

        let player_index = message.get_player_index();

        if self.client_addresses.len() <= player_index ||
            self.client_addresses[player_index].is_none() ||
            !self.client_addresses[player_index].as_ref().unwrap().get_ip_address().eq(&source.ip()) {

            warn!("Received a message from an unexpected source. player_index: {:?}, source: {:?}",
                  player_index, source.ip());
            return;
        }

        self.handle_remote_peer(message.get_player_index(), source);

        match message {
            ToServerMessageUDP::Hello { player_index } => {

            }
            ToServerMessageUDP::Input(input_message) => {
                self.input_consumers.accept(&input_message);
            }
        }
    }

    fn handle_remote_peer(&mut self, player_index: usize, remote_peer: SocketAddr) {
        while self.remote_peers.len() <= player_index {
            self.remote_peers.push(None);
        }

        let remote_peer = RemoteUdpPeer::new(player_index, remote_peer);

        match &self.remote_peers[player_index] {
            None => {
                info!("First time UDP remote peer: {:?}", remote_peer);
                self.remote_peers[player_index] = Some(remote_peer);
                self.remote_peer_consumers.accept(self.remote_peers[player_index].as_ref().unwrap());
            }
            Some(existing_remote_peer) => {
                let existing_socket = existing_remote_peer.get_socket_addr();
                if !existing_socket.eq(&remote_peer.get_socket_addr()) {
                    info!("Change of UDP remote peer: {:?}", remote_peer);
                    self.remote_peers[player_index] = Some(remote_peer);
                    self.remote_peer_consumers.accept(self.remote_peers[player_index].as_ref().unwrap());
                    self.fragment_assemblers.remove(&existing_socket);
                }
            }
        }
    }
}

impl<InputType: Input> ChannelThread<()> for UdpInput<InputType> {

    fn run(mut self, mut receiver: Receiver<Self>) -> () {

        info!("Starting.");

        loop {

            let mut buf = [0; MAX_UDP_DATAGRAM_SIZE];
            //TODO: check source against valid sources
            let (number_of_bytes, source) = self.socket.recv_from(&mut buf).unwrap();
            let filled_buf = &mut buf[..number_of_bytes];

            receiver.try_iter(&mut self);

            self.handle_receive(filled_buf, source);
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

impl<InputType: Input> Consumer<ClientAddress> for Sender<UdpInput<InputType>> {
    fn accept(&self, client_address: ClientAddress) {
        self.send(move |udp_input|{
            udp_input.client_ip_set.insert(client_address.get_ip_address());

            let index = client_address.get_player_index();
            while udp_input.client_addresses.len() <= index {
                udp_input.client_addresses.push(None);
            }

            udp_input.client_addresses[index] = Some(client_address);

            info!("Added Client: {:?}", udp_input.client_addresses[index].as_ref().unwrap());
        }).unwrap();
    }
}



