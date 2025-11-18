use crate::real_time::net::tcp::{
    TcpReader,
    TcpStream,
};
use crate::real_time::net::udp::UdpSocket;
use crate::real_time::net::LOCAL_EPHEMERAL_SOCKET_ADDR_V4;
use crate::real_time::real::RealFactory;
use crate::real_time::simulation::SingleThreadedFactory;
use crate::real_time::{
    Receiver,
    Sender,
    TimeSource,
};
use std::io::Error;
use std::net::SocketAddr;

#[derive(Clone)]
pub enum FactoryImplementation {
    Real(RealFactory),
    Simulated(SingleThreadedFactory),
}

#[derive(Clone)]
pub struct Factory {
    pub(super) implementation: FactoryImplementation,
}

impl Factory {
    pub fn new() -> Self {
        Self {
            implementation: FactoryImplementation::Real(RealFactory::new()),
        }
    }

    pub fn get_time_source(&self) -> &TimeSource {
        match &self.implementation {
            FactoryImplementation::Real(real_factory) => real_factory.get_time_source(),
            FactoryImplementation::Simulated(single_threaded_factory) => {
                single_threaded_factory.get_time_source()
            }
        }
    }

    pub fn new_channel<T: Send>(&self) -> (Sender<T>, Receiver<T>) {
        match &self.implementation {
            FactoryImplementation::Real(real_factory) => real_factory.new_channel(),
            FactoryImplementation::Simulated(single_threaded_factory) => {
                single_threaded_factory.new_channel()
            }
        }
    }

    pub fn connect_tcp(&self, socket_addr: SocketAddr) -> Result<(TcpStream, TcpReader), Error> {
        match &self.implementation {
            FactoryImplementation::Real(real_factory) => real_factory.connect_tcp(socket_addr),
            FactoryImplementation::Simulated(single_threaded_factory) => {
                single_threaded_factory.connect_tcp(socket_addr)
            }
        }
    }

    pub fn bind_udp_socket(&self, socket_addr: SocketAddr) -> Result<UdpSocket, Error> {
        match &self.implementation {
            FactoryImplementation::Real(real_factory) => real_factory.bind_udp_socket(socket_addr),
            FactoryImplementation::Simulated(single_threaded_factory) => {
                single_threaded_factory.bind_udp_socket(socket_addr)
            }
        }
    }

    pub fn bind_udp_ephemeral_port(&self) -> Result<UdpSocket, Error> {
        return self.bind_udp_socket(SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4));
    }
}
