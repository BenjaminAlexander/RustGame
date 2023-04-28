use std::io::Error;
use std::net::SocketAddr;

pub trait UdpSocketTrait: Send + Sized {

    fn send_to(&mut self, buf: &[u8], socket_addr: &SocketAddr) -> Result<usize, Error>;

    fn try_clone(&self) -> Result<Self, Error>;
}