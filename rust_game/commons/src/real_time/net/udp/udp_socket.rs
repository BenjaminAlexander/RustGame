use std::{
    io::Error,
    net::SocketAddr,
};

use crate::real_time::{
    real::net::udp::RealUdpSocket,
    simulation::net::udp::UdpSocketSimulator,
};

pub(super) enum UdpSocketImplementation {
    Real(RealUdpSocket),
    Simulated(UdpSocketSimulator),
}

pub struct UdpSocket {
    implementation: UdpSocketImplementation,
}

impl UdpSocket {
    pub fn new(real_udp_socket: RealUdpSocket) -> Self {
        return Self {
            implementation: UdpSocketImplementation::Real(real_udp_socket),
        };
    }

    pub fn new_simulated(udp_socket_simulator: UdpSocketSimulator) -> Self {
        return Self {
            implementation: UdpSocketImplementation::Simulated(udp_socket_simulator),
        };
    }

    pub(super) fn take_implementation(self) -> UdpSocketImplementation {
        return self.implementation;
    }

    pub fn local_addr(&self) -> Result<SocketAddr, Error> {
        match &self.implementation {
            UdpSocketImplementation::Real(real_udp_socket) => real_udp_socket.local_addr(),
            UdpSocketImplementation::Simulated(udp_socket_simulator) => {
                Ok(udp_socket_simulator.local_addr())
            }
        }
    }

    pub fn send_to(&mut self, buf: &[u8], socket_addr: &SocketAddr) -> Result<usize, Error> {
        match &mut self.implementation {
            UdpSocketImplementation::Real(real_udp_socket) => real_udp_socket.send_to(buf, socket_addr),
            UdpSocketImplementation::Simulated(udp_socket_simulator) => {
                udp_socket_simulator.send_to(buf, socket_addr)
            }
        }
    }

    pub fn try_clone(&self) -> Result<Self, Error> {
        match &self.implementation {
            UdpSocketImplementation::Real(real_udp_socket) => Ok(Self {
                implementation: UdpSocketImplementation::Real(real_udp_socket.try_clone()?),
            }),
            UdpSocketImplementation::Simulated(udp_socket_simulator) => Ok(Self {
                implementation: UdpSocketImplementation::Simulated(udp_socket_simulator.try_clone()?),
            }),
        }
    }
}
