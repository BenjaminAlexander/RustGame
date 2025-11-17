use crate::interface::GameFactoryTrait;
use crate::messaging::{
    FragmentAssembler,
    InputMessage,
    MessageFragment,
    ToServerMessageUDP,
};
use crate::server::clientaddress::ClientAddress;
use crate::server::remoteudppeer::RemoteUdpPeer;
use commons::real_time::net::MAX_UDP_DATAGRAM_SIZE;
use log::{
    info,
    warn,
};
use std::collections::{
    HashMap,
    HashSet,
};
use std::net::{
    IpAddr,
    SocketAddr,
};

pub struct UdpHandler<GameFactory: GameFactoryTrait> {
    factory: GameFactory::Factory,
    remote_peers: Vec<Option<RemoteUdpPeer>>,
    client_addresses: Vec<Option<ClientAddress>>,
    client_ip_set: HashSet<IpAddr>,

    //TODO: timeout fragments or fragment assemblers
    fragment_assemblers: HashMap<SocketAddr, FragmentAssembler>,
}

impl<GameFactory: GameFactoryTrait> UdpHandler<GameFactory> {
    pub fn new(factory: GameFactory::Factory) -> Self {
        return Self {
            factory,
            remote_peers: Vec::new(),
            client_addresses: Vec::new(),
            client_ip_set: HashSet::new(),
            //TODO: make this more configurable
            fragment_assemblers: HashMap::new(),
        };
    }

    pub fn on_client_address(&mut self, client_address: ClientAddress) {
        self.client_ip_set.insert(client_address.get_ip_address());

        let index = client_address.get_player_index();
        while self.client_addresses.len() <= index {
            self.client_addresses.push(None);
        }

        self.client_addresses[index] = Some(client_address);

        info!(
            "Added Client: {:?}",
            self.client_addresses[index].as_ref().unwrap()
        );
    }

    pub fn on_udp_packet(
        &mut self,
        number_of_bytes: usize,
        mut buf: [u8; MAX_UDP_DATAGRAM_SIZE],
        source: SocketAddr,
    ) -> (
        Option<RemoteUdpPeer>,
        Option<InputMessage<GameFactory::Game>>,
    ) {
        //TODO: check source against valid sources
        let mut filled_buf = &mut buf[..number_of_bytes];

        if !self.client_ip_set.contains(&source.ip()) {
            warn!("Unexpected UDP packet received from {:?}", source);
            return (None, None);
        }

        if let Some(assembled) = self.handle_fragment(source, &mut filled_buf) {
            match rmp_serde::from_slice(assembled.as_slice()) {
                Ok(message) => {
                    return self.handle_message(message, source);
                }
                Err(error) => {
                    //TODO: is removing the fragement assembler on error right?
                    self.fragment_assemblers.remove(&source);
                    warn!("Failed to deserialize a ToServerMessageUDP: {:?}", error);
                    return (None, None);
                }
            }
        } else {
            return (None, None);
        }
    }

    fn handle_fragment(&mut self, source: SocketAddr, fragment: &mut [u8]) -> Option<Vec<u8>> {
        let assembler = match self.fragment_assemblers.get_mut(&source) {
            None => {
                //TODO: make max_messages more configurable
                self.fragment_assemblers
                    .insert(source, FragmentAssembler::new(&self.factory, 5));
                self.fragment_assemblers.get_mut(&source).unwrap()
            }
            Some(assembler) => assembler,
        };

        return assembler.add_fragment(MessageFragment::from_vec(fragment.to_vec()));
    }

    fn handle_message(
        &mut self,
        message: ToServerMessageUDP<GameFactory::Game>,
        source: SocketAddr,
    ) -> (
        Option<RemoteUdpPeer>,
        Option<InputMessage<GameFactory::Game>>,
    ) {
        let player_index = message.get_player_index();

        if self.client_addresses.len() <= player_index
            || self.client_addresses[player_index].is_none()
            || !self.client_addresses[player_index]
                .as_ref()
                .unwrap()
                .get_ip_address()
                .eq(&source.ip())
        {
            warn!(
                "Received a message from an unexpected source. player_index: {:?}, source: {:?}",
                player_index,
                source.ip()
            );
            return (None, None);
        }

        let remote_peer = self.handle_remote_peer(message.get_player_index(), source);

        match message {
            ToServerMessageUDP::Hello { player_index: _ } => {
                return (remote_peer, None);
            }
            ToServerMessageUDP::Input(input_message) => {
                return (remote_peer, Some(input_message));
            }
        }
    }

    fn handle_remote_peer(
        &mut self,
        player_index: usize,
        remote_peer: SocketAddr,
    ) -> Option<RemoteUdpPeer> {
        while self.remote_peers.len() <= player_index {
            self.remote_peers.push(None);
        }

        let remote_peer = RemoteUdpPeer::new(player_index, remote_peer);

        match &self.remote_peers[player_index] {
            None => {
                info!("First time UDP remote peer: {:?}", remote_peer);
                self.remote_peers[player_index] = Some(remote_peer.clone());
                return Some(remote_peer);
            }
            Some(existing_remote_peer) => {
                let existing_socket = existing_remote_peer.get_socket_addr();
                if !existing_socket.eq(&remote_peer.get_socket_addr()) {
                    info!("Change of UDP remote peer: {:?}", remote_peer);
                    self.remote_peers[player_index] = Some(remote_peer.clone());
                    self.fragment_assemblers.remove(&existing_socket);
                    return Some(remote_peer);
                } else {
                    return None;
                }
            }
        }
    }
}
