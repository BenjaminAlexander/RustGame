use std::io::Error;
use std::net::{SocketAddr, UdpSocket};
use crate::net::UdpSocketTrait;

pub struct RealUdpSocket {
    udp_socket: UdpSocket
}

impl RealUdpSocket {

    pub fn bind(socket_addr: SocketAddr) -> Result<Self, Error> {
        return Ok(Self {
            udp_socket: UdpSocket::bind(socket_addr)?
        });
    }

    pub fn recv_from(&mut self, buf: &mut [u8]) -> Result<(usize, SocketAddr), Error> {
        return self.udp_socket.recv_from(buf);
    }

}

impl UdpSocketTrait for RealUdpSocket {
    fn send_to(&mut self, buf: &[u8], socket_addr: &SocketAddr) -> Result<usize, Error> {
        return self.udp_socket.send_to(buf, socket_addr);
    }

    fn try_clone(&self) -> Result<Self, Error> {
        return Ok(Self {
            udp_socket: self.udp_socket.try_clone()?
        });
    }
}