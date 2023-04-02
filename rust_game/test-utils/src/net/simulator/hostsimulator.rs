use std::io::Error;
use std::marker::PhantomData;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use commons::factory::FactoryTrait;
use crate::net::{ChannelTcpReader, ChannelTcpWriter, NetworkSimulator};
use crate::singlethreaded::SingleThreadedFactory;

#[derive(Clone)]
pub struct HostSimulator {
    ip_addr: Arc<IpAddr>,
    network_simulator: NetworkSimulator
}

impl HostSimulator {
    pub fn new(network_simulator: NetworkSimulator, ip_addr: IpAddr) -> Self {
        return Self {
            network_simulator,
            ip_addr: Arc::new(ip_addr)
        };
    }

    pub fn get_ip_addr(&self) -> IpAddr {
        return *self.ip_addr;
    }

    pub fn get_network_simulator(&self) -> &NetworkSimulator {
        return &self.network_simulator;
    }

    fn connect_tcp(&self, socket_addr: SocketAddr) -> Result<(ChannelTcpWriter<SingleThreadedFactory>, ChannelTcpReader<SingleThreadedFactory>), Error> {
        todo!()
    }
}