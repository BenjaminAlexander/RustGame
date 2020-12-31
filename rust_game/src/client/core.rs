use std::net::{Ipv4Addr, SocketAddrV4, SocketAddr, TcpStream};
use std::str::FromStr;
use crate::gametime::{TimeDuration, GameTimer};
use crate::threading::ChannelThread;

pub struct Core {

}

impl Core {
    pub fn new(server_ip: &str,
               port: u16,
               step_duration: TimeDuration,
               clock_average_size: usize) -> Self {

        let ip_addr_v4 = Ipv4Addr::from_str(server_ip).unwrap();
        let socket_addr_v4 = SocketAddrV4::new(ip_addr_v4, port);
        let socket_addr:SocketAddr = SocketAddr::from(socket_addr_v4);
        let tcp_stream = TcpStream::connect(socket_addr);

        let (game_timer_sender, game_timer_builder) = GameTimer::new(step_duration, clock_average_size).build();
        let game_timer_join_handle = game_timer_builder.name("ClientGameTimer").start().unwrap();

        Core{}
    }
}