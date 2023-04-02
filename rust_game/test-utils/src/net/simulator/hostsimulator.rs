use std::io::Error;
use std::marker::PhantomData;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use commons::factory::FactoryTrait;
use crate::net::{ChannelTcpReader, ChannelTcpWriter, NetworkSimulator};

#[derive(Clone)]
pub struct HostSimulator<Factory: FactoryTrait<TcpWriter=ChannelTcpWriter<Factory>, TcpReader=ChannelTcpReader<Factory>>> {
    ip_addr: Arc<IpAddr>,
    network_simulator: NetworkSimulator<Factory>
}

impl<Factory: FactoryTrait<TcpWriter=ChannelTcpWriter<Factory>, TcpReader=ChannelTcpReader<Factory>>> HostSimulator<Factory> {
    pub fn new(network_simulator: NetworkSimulator<Factory>, ip_addr: IpAddr) -> Self {
        return Self {
            network_simulator,
            ip_addr: Arc::new(ip_addr)
        };
    }

    pub fn get_ip_addr(&self) -> IpAddr {
        return *self.ip_addr;
    }

    pub fn get_network_simulator(&self) -> &NetworkSimulator<Factory> {
        return &self.network_simulator;
    }

    fn connect_tcp(&self, socket_addr: SocketAddr) -> Result<(ChannelTcpWriter<Factory>, ChannelTcpReader<Factory>), Error> {
        todo!()
    }
}