use crate::net::{
    UdpReadHandlerTrait,
    UdpSocketTrait,
};
use crate::real_time::{EventOrStopThread, Receiver};
use std::io::Error;
use std::net::{
    SocketAddr,
    UdpSocket,
};

use super::NET_POLLING_PERIOD;

pub struct RealUdpSocket {
    udp_socket: UdpSocket,
}

impl RealUdpSocket {
    pub fn bind(socket_addr: SocketAddr) -> Result<Self, Error> {
        let udp_socket = UdpSocket::bind(socket_addr)?;
        udp_socket.set_read_timeout(Some(NET_POLLING_PERIOD.to_duration().unwrap()))?;

        return Ok(Self { udp_socket });
    }

    pub fn recv_from(&mut self, buf: &mut [u8]) -> Result<(usize, SocketAddr), Error> {
        return self.udp_socket.recv_from(buf);
    }

    pub fn local_addr(&self) -> Result<SocketAddr, Error> {
        return self.udp_socket.local_addr();
    }

    pub fn spawn_real_udp_reader<T: UdpReadHandlerTrait>(
        self,
        thread_name: String,
        receiver: Receiver<EventOrStopThread<()>>,
        udp_read_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> Result<(), Error> {
        return receiver.spawn_real_udp_reader(thread_name, self, udp_read_handler, join_call_back);
    }
}

impl UdpSocketTrait for RealUdpSocket {
    fn send_to(&mut self, buf: &[u8], socket_addr: &SocketAddr) -> Result<usize, Error> {
        return self.udp_socket.send_to(buf, socket_addr);
    }

    fn try_clone(&self) -> Result<Self, Error> {
        return Ok(Self {
            udp_socket: self.udp_socket.try_clone()?,
        });
    }
}
