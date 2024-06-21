use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct RemoteUdpPeer {
    player_index: usize,
    remote_peer: SocketAddr,
}

impl RemoteUdpPeer {
    pub fn new(player_index: usize, remote_peer: SocketAddr) -> Self {
        return Self {
            player_index,
            remote_peer,
        };
    }

    pub fn get_player_index(&self) -> usize {
        return self.player_index;
    }

    pub fn get_socket_addr(&self) -> SocketAddr {
        return self.remote_peer;
    }
}
