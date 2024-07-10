use std::net::{
    Ipv4Addr,
    SocketAddrV4,
};

use crate::time::TimeDuration;

pub const MAX_UDP_DATAGRAM_SIZE: usize = 1500;

pub const LOCAL_IP_V4: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
pub const LOCAL_EPHEMERAL_SOCKET_ADDR_V4: SocketAddrV4 = SocketAddrV4::new(LOCAL_IP_V4, 0);

pub const TCP_LISTENER_POLLING_PERIOD: TimeDuration = TimeDuration::ONE_SECOND;
