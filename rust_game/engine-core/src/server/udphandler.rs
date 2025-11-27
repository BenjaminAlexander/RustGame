use crate::messaging::{
    FragmentAssembler,
    FragmentableUdpToServerMessage,
    MessageFragment,
};
use crate::server::clientaddress::ClientAddress;
use crate::server::remoteudppeer::RemoteUdpPeer;
use crate::GameTrait;
use commons::real_time::TimeSource;
use log::{
    info,
    warn,
};
use std::collections::{
    HashMap,
    HashSet,
};
use std::marker::PhantomData;
use std::net::{
    IpAddr,
    SocketAddr,
};

//TODO: This struct could be combined with server udp input
pub struct UdpHandler<Game: GameTrait> {
    time_source: TimeSource,
    remote_peers: Vec<Option<RemoteUdpPeer>>,
    client_addresses: Vec<Option<ClientAddress>>,
    client_ip_set: HashSet<IpAddr>,

    //TODO: timeout fragments or fragment assemblers
    fragment_assemblers: HashMap<SocketAddr, FragmentAssembler>,
    phantom: PhantomData<Game>,
}

impl<Game: GameTrait> UdpHandler<Game> {
    pub fn new(time_source: TimeSource) -> Self {
        return Self {
            time_source,
            remote_peers: Vec::new(),
            client_addresses: Vec::new(),
            client_ip_set: HashSet::new(),
            //TODO: make this more configurable
            fragment_assemblers: HashMap::new(),
            phantom: PhantomData,
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
        buf: &[u8],
        source: SocketAddr,
    ) -> (Option<RemoteUdpPeer>, Option<FragmentableUdpToServerMessage<Game>>) {
        //TODO: check source against valid sources

        if !self.client_ip_set.contains(&source.ip()) {
            warn!("Unexpected UDP packet received from {:?}", source);
            return (None, None);
        }

        if let Some(assembled) = self.handle_fragment(source, &buf) {
            match rmp_serde::from_slice(assembled.as_slice()) {
                Ok(message) => {
                    return (self.handle_message(&message, source), Some(message));
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

    fn handle_fragment(&mut self, source: SocketAddr, fragment: &[u8]) -> Option<Vec<u8>> {
        let assembler = match self.fragment_assemblers.get_mut(&source) {
            None => {
                //TODO: make max_messages more configurable
                self.fragment_assemblers
                    .insert(source, FragmentAssembler::new(self.time_source.clone(), 5));
                self.fragment_assemblers.get_mut(&source).unwrap()
            }
            Some(assembler) => assembler,
        };

        return assembler.add_fragment(MessageFragment::from_vec(fragment.to_vec()));
    }

    fn handle_message(
        &mut self,
        message: &FragmentableUdpToServerMessage<Game>,
        source: SocketAddr,
    ) -> Option<RemoteUdpPeer> {
        let player_index = message.get_player_index();

        let source_is_valid = match self.client_addresses.get(player_index) {
            Some(Some(expected_source)) if expected_source.get_ip_address().eq(&source.ip()) => {
                true
            }
            _ => false,
        };

        if !source_is_valid {
            warn!(
                "Received a message from an unexpected source. player_index: {:?}, source: {:?}",
                player_index,
                source.ip()
            );
            return None;
        }

        return self.handle_remote_peer(message.get_player_index(), source);
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
