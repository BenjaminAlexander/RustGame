use std::{
    io::Error,
    net::SocketAddr,
};

use crate::{
    net::{
        RealUdpSocket,
        UdpReadHandlerTrait,
        UdpSocketTrait,
    },
    single_threaded_simulator::net::UdpSocketSimulator,
    threading::{
        channel::Receiver,
        eventhandling::EventOrStopThread,
        AsyncJoinCallBackTrait,
        ThreadBuilder,
    },
};

enum Implementation {
    Real(RealUdpSocket),
    Simulated(UdpSocketSimulator),
}

pub struct UdpSocket {
    implementation: Implementation,
}

impl UdpSocket {
    pub fn new(real_udp_socket: RealUdpSocket) -> Self {
        return Self {
            implementation: Implementation::Real(real_udp_socket),
        };
    }

    pub fn new_simulated(udp_socket_simulator: UdpSocketSimulator) -> Self {
        return Self {
            implementation: Implementation::Simulated(udp_socket_simulator),
        };
    }

    pub fn local_addr(&self) -> Result<SocketAddr, Error> {
        match &self.implementation {
            Implementation::Real(real_udp_socket) => real_udp_socket.local_addr(),
            Implementation::Simulated(udp_socket_simulator) => {
                Ok(udp_socket_simulator.local_addr())
            }
        }
    }

    pub fn send_to(&mut self, buf: &[u8], socket_addr: &SocketAddr) -> Result<usize, Error> {
        match &mut self.implementation {
            Implementation::Real(real_udp_socket) => real_udp_socket.send_to(buf, socket_addr),
            Implementation::Simulated(udp_socket_simulator) => {
                udp_socket_simulator.send_to(buf, socket_addr)
            }
        }
    }

    pub fn try_clone(&self) -> Result<Self, Error> {
        match &self.implementation {
            Implementation::Real(real_udp_socket) => Ok(Self {
                implementation: Implementation::Real(real_udp_socket.try_clone()?),
            }),
            Implementation::Simulated(udp_socket_simulator) => Ok(Self {
                implementation: Implementation::Simulated(udp_socket_simulator.try_clone()?),
            }),
        }
    }

    pub fn spawn_udp_reader<T: UdpReadHandlerTrait>(
        self,
        thread_builder: ThreadBuilder,
        receiver: Receiver<EventOrStopThread<()>>,
        udp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T>,
    ) -> Result<(), Error> {
        match self.implementation {
            Implementation::Real(real_udp_socket) => real_udp_socket.spawn_real_udp_reader(
                thread_builder,
                receiver,
                udp_read_handler,
                join_call_back,
            ),
            Implementation::Simulated(udp_socket_simulator) => udp_socket_simulator
                .spawn_simulated_udp_reader(
                    thread_builder,
                    receiver,
                    udp_read_handler,
                    join_call_back,
                ),
        }
    }
}
