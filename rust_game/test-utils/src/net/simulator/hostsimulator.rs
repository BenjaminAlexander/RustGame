use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use crate::net::NetworkSimulator;

#[derive(Clone)]
pub struct HostSimulator {
    ip_addr: Arc<Mutex<IpAddr>>,
    network_simulator: NetworkSimulator
}

impl HostSimulator {
    pub fn new(network_simulator: NetworkSimulator, ip_addr: IpAddr) -> Self {
        return Self {
            network_simulator,
            ip_addr: Arc::new(Mutex::new(ip_addr))
        };
    }

    pub fn get_ip_addr(&self) -> IpAddr {
        return *self.ip_addr.lock().unwrap();
    }

    pub fn set_ip_addr(&mut self, ip_addr: IpAddr) {
        *self.ip_addr.lock().unwrap() = ip_addr;
    }

    pub fn get_network_simulator(&self) -> &NetworkSimulator {
        return &self.network_simulator;
    }
}