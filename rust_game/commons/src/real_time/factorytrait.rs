use crate::net::{
    TcpStream,
    UdpSocket,
    LOCAL_EPHEMERAL_SOCKET_ADDR_V4,
};
use crate::real_time::net::tcp::TcpReader;
use crate::real_time::{Receiver, Sender, TimeSource};
use std::io::Error;
use std::net::SocketAddr;

//TODO: rename trait and file
pub trait FactoryTrait: Clone + Send + 'static {
    fn get_time_source(&self) -> &TimeSource;
    
    fn new_channel<T: Send>(&self) -> (Sender<T>, Receiver<T>);

    fn connect_tcp(&self, socket_addr: SocketAddr) -> Result<(TcpStream, TcpReader), Error>;

    fn bind_udp_socket(&self, socket_addr: SocketAddr) -> Result<UdpSocket, Error>;

    fn bind_udp_ephemeral_port(&self) -> Result<UdpSocket, Error> {
        return self.bind_udp_socket(SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4));
    }
}
