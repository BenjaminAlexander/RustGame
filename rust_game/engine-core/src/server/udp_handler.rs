use {
    crate::{
        Play,
        messaging::{
            UdpStreamReceiver, UdpToServerMessage
        },
        server::{
            player_address::PlayerAddress,
            player_udp_address::PlayerUdpAddress,
        },
    }, commons::real_time::net::MAX_UDP_DATAGRAM_SIZE, log::{
        info,
        warn,
    }, std::{
        collections::{
            HashMap,
            HashSet,
        },
        marker::PhantomData,
        net::{
            IpAddr,
            SocketAddr,
        },
    }
};

//TODO: This struct could be combined with server udp input
pub struct UdpHandler<Game: Play> {
    remote_peers: Vec<Option<PlayerUdpAddress>>,
    client_addresses: Vec<Option<PlayerAddress>>,
    client_ip_set: HashSet<IpAddr>,

    //TODO: timeout fragments or fragment assemblers
    //TODO: rename
    fragment_assemblers: HashMap<SocketAddr, UdpStreamReceiver>,
    phantom: PhantomData<Game>,
}

impl<Game: Play> UdpHandler<Game> {
    pub fn new() -> Self {
        return Self {
            remote_peers: Vec::new(),
            client_addresses: Vec::new(),
            client_ip_set: HashSet::new(),
            //TODO: make this more configurable
            fragment_assemblers: HashMap::new(),
            phantom: PhantomData,
        };
    }

    pub fn on_client_address(&mut self, client_address: PlayerAddress) {
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
    ) -> (Option<PlayerUdpAddress>, Option<UdpToServerMessage<Game>>) {
        //TODO: check source against valid sources

        if !self.client_ip_set.contains(&source.ip()) {
            warn!("Unexpected UDP packet received from {:?}", source);
            return (None, None);
        }

        if let Some(message) = self.handle_fragment(source, &buf) {
            return (self.handle_message(&message, source), Some(message));
        } else {
            return (None, None);
        }
    }

    fn handle_fragment(&mut self, source: SocketAddr, fragment: &[u8]) -> Option<UdpToServerMessage<Game>> {
        let assembler = match self.fragment_assemblers.get_mut(&source) {
            None => {
                //TODO: make max_messages more configurable
                self.fragment_assemblers
                    .insert(source, UdpStreamReceiver::new(MAX_UDP_DATAGRAM_SIZE, 5));
                self.fragment_assemblers.get_mut(&source).unwrap()
            }
            Some(assembler) => assembler,
        };

        return assembler.receive(fragment);
    }

    fn handle_message(
        &mut self,
        message: &UdpToServerMessage<Game>,
        source: SocketAddr,
    ) -> Option<PlayerUdpAddress> {
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
    ) -> Option<PlayerUdpAddress> {
        while self.remote_peers.len() <= player_index {
            self.remote_peers.push(None);
        }

        let remote_peer = PlayerUdpAddress::new(player_index, remote_peer);

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
