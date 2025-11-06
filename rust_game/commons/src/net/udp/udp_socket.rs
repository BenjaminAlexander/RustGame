use std::{io::Error, net::SocketAddr};

use crate::{net::{RealUdpSocket, UdpSocketTrait}, single_threaded_simulator::net::UdpSocketSimulator};


enum Implementation {
    Real(RealUdpSocket),
    Simulated(UdpSocketSimulator)
}

pub struct UdpSocket {
    implementation: Implementation
}


impl UdpSocket {

    pub fn send_to(&mut self, buf: &[u8], socket_addr: &SocketAddr) -> Result<usize, Error> {
        match &mut self.implementation {
            Implementation::Real(real_udp_socket) => real_udp_socket.send_to(buf, socket_addr),
            Implementation::Simulated(udp_socket_simulator) => udp_socket_simulator.send_to(buf, socket_addr),
        }
    }

    pub fn try_clone(&self) -> Result<Self, Error> {
        match &self.implementation {
            Implementation::Real(real_udp_socket) => Ok(Self { 
                implementation: Implementation::Real(real_udp_socket.try_clone()?)
            }),
            Implementation::Simulated(udp_socket_simulator) => Ok(Self { 
                implementation: Implementation::Simulated(udp_socket_simulator.try_clone()?)
            }),
        }
    }
}