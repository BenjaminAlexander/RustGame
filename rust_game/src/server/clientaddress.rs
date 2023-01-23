use std::net::IpAddr;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ClientAddress {
    player_index: usize,
    ip_address: IpAddr
}

impl ClientAddress {
    pub fn new(player_index: usize, ip_address: IpAddr) -> Self {
        return Self{
            player_index,
            ip_address
        };
    }

    pub fn get_ip_address(&self) -> IpAddr {
        return self.ip_address;
    }

    pub fn get_player_index(&self) -> usize {
        return self.player_index;
    }
}