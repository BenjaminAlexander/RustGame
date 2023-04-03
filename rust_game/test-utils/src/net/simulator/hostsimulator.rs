use std::io::Error;
use std::marker::PhantomData;
use std::net::{IpAddr, SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex};
use commons::factory::FactoryTrait;
use crate::net::{ChannelTcpReader, ChannelTcpWriter, NetworkSimulator};
use crate::singlethreaded::SingleThreadedFactory;

#[derive(Clone)]
pub struct HostSimulator {
    ip_addr: Arc<IpAddr>,
    next_port: Arc<Mutex<u16>>,
    network_simulator: NetworkSimulator
}

impl HostSimulator {
    pub fn new(network_simulator: NetworkSimulator, ip_addr: IpAddr) -> Self {
        return Self {
            network_simulator,
            ip_addr: Arc::new(ip_addr),
            next_port: Arc::new(Mutex::new(1))
        };
    }

    pub fn get_ip_addr(&self) -> IpAddr {
        return *self.ip_addr;
    }

    pub fn get_network_simulator(&self) -> &NetworkSimulator {
        return &self.network_simulator;
    }

    pub fn connect_tcp(&self, factory: &SingleThreadedFactory, server_socket_addr: SocketAddr) -> Result<(ChannelTcpWriter, ChannelTcpReader), Error> {

        let port;
        {
            let mut guard = self.next_port.lock().unwrap();
            port = *guard;
            *guard = *guard + 1;
        }

        let ip_addr = self.ip_addr.as_ref().clone();
        let client_socket_addr = SocketAddr::new(ip_addr, port);

        return self.network_simulator.connect_tcp(factory, client_socket_addr, server_socket_addr);
    }
}