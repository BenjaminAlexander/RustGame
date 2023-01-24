use log::{info, warn};
use crate::messaging::{MAX_UDP_DATAGRAM_SIZE, ToServerMessageUDP, FragmentAssembler, MessageFragment};
use crate::threading::ChannelDrivenThreadSender;
use crate::interface::GameTrait;
use std::net::{UdpSocket, SocketAddr, IpAddr};
use std::io;
use crate::server::remoteudppeer::RemoteUdpPeer;
use std::collections::{HashMap, HashSet};
use std::ops::ControlFlow::{Break, Continue};
use crate::server::clientaddress::ClientAddress;
use crate::server::ServerCore;
use crate::threading::channel::ReceiveMetaData;
use crate::threading::listener::{ChannelEvent, ListenerEventResult, ListenerTrait, ListenResult};
use crate::threading::listener::ListenedOrDidNotListen::{DidNotListen, Listened};

//TODO: timeout fragments or fragment assemblers

#[derive(Debug)]
pub enum UdpInputEvent {
    ClientAddress(ClientAddress)
}

pub struct UdpInput<Game: GameTrait> {
    socket: UdpSocket,
    remote_peers: Vec<Option<RemoteUdpPeer>>,
    client_addresses: Vec<Option<ClientAddress>>,
    client_ip_set: HashSet<IpAddr>,
    fragment_assemblers: HashMap<SocketAddr, FragmentAssembler>,
    core_sender: ChannelDrivenThreadSender<ServerCore<Game>>
}

impl<Game: GameTrait> UdpInput<Game> {

    pub fn new(
        socket: &UdpSocket,
        core_sender: ChannelDrivenThreadSender<ServerCore<Game>>) -> io::Result<Self> {
        return Ok(Self {
            socket: socket.try_clone()?,
            remote_peers: Vec::new(),
            client_addresses: Vec::new(),
            client_ip_set: HashSet::new(),
            //TODO: make this more configurable
            fragment_assemblers: HashMap::new(),
            core_sender
        });
    }

    fn channel_empty_After_listen(&mut self, mut buf: [u8; MAX_UDP_DATAGRAM_SIZE], number_of_bytes: usize, source: SocketAddr) {
        //TODO: check source against valid sources
        let filled_buf = &mut buf[..number_of_bytes];

        if !self.client_ip_set.contains(&source.ip()) {
            warn!("Unexpected UDP packet received from {:?}", source);
            return;
        }

        if let Some(assembled) = self.handle_fragment(source, filled_buf) {
            match rmp_serde::from_read_ref(assembled.as_slice()) {
                Ok(message) => {
                    self.handle_message(message, source);
                }
                Err(error) => {

                    //TODO: is removing the fragement assembler on error right?
                    self.fragment_assemblers.remove(&source);
                    warn!("Failed to deserialize a ToServerMessageUDP: {:?}", error);
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

    fn handle_message(&mut self, message: ToServerMessageUDP<Game>, source: SocketAddr) {

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
            ToServerMessageUDP::Hello {player_index: _} => {

            }
            ToServerMessageUDP::Input(input_message) => {
                self.core_sender.on_input_message(input_message);
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
                self.core_sender.on_remote_udp_peer(remote_peer.clone());
                self.remote_peers[player_index] = Some(remote_peer);
            }
            Some(existing_remote_peer) => {
                let existing_socket = existing_remote_peer.get_socket_addr();
                if !existing_socket.eq(&remote_peer.get_socket_addr()) {
                    info!("Change of UDP remote peer: {:?}", remote_peer);
                    self.core_sender.on_remote_udp_peer(remote_peer.clone());
                    self.remote_peers[player_index] = Some(remote_peer);
                    self.fragment_assemblers.remove(&existing_socket);
                }
            }
        }
    }
}

impl<Game: GameTrait> ListenerTrait for UdpInput<Game> {
    type Event = UdpInputEvent;
    type ThreadReturn = ();
    type ListenFor = ([u8; MAX_UDP_DATAGRAM_SIZE], usize, SocketAddr);

    fn listen(mut self) -> ListenResult<Self> {
        let mut buf = [0; MAX_UDP_DATAGRAM_SIZE];

        let recv_result = self.socket.recv_from(&mut buf);

        match recv_result {
            Ok((number_of_bytes, source)) => {
                return Continue(Listened(self, (buf, number_of_bytes, source)));
            }
            Err(e) => {
                warn!("Error: {:?}", e);
                return Continue(DidNotListen(self));
            }
        }
    }

    fn on_channel_event(mut self, event: ChannelEvent<Self>) -> ListenerEventResult<Self> {
        match event {
            ChannelEvent::ChannelEmptyAfterListen(listened_value_holder) => {
                let (buf, number_of_bytes, source) = listened_value_holder.move_value();
                self.channel_empty_After_listen(buf, number_of_bytes, source);
                return Continue(self);
            }
            ChannelEvent::ReceivedEvent(_, event) => {
                match event {
                    UdpInputEvent::ClientAddress(client_address) => {
                        self.client_ip_set.insert(client_address.get_ip_address());

                        let index = client_address.get_player_index();
                        while self.client_addresses.len() <= index {
                            self.client_addresses.push(None);
                        }

                        self.client_addresses[index] = Some(client_address);

                        info!("Added Client: {:?}", self.client_addresses[index].as_ref().unwrap());

                        return Continue(self);
                    }
                }
            }
            ChannelEvent::ChannelDisconnected => Break(())
        }
    }

    fn on_stop(self, _: ReceiveMetaData) -> Self::ThreadReturn { () }
}