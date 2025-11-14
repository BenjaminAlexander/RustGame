use crate::real_time::net::tcp::{TcpReader, TcpStream};
use crate::real_time::simulation::SingleThreadedFactory;
use crate::real_time::simulation::net::network_simulator::NetworkSimulator;
use crate::real_time::simulation::net::udp::UdpSocketSimulator;
use std::collections::HashSet;
use std::io::{
    Error,
    ErrorKind,
};
use std::net::{
    IpAddr,
    SocketAddr,
};
use std::sync::{
    Arc,
    Mutex,
};

#[derive(Clone)]
pub struct HostSimulator {
    internal: Arc<Mutex<Internal>>,
    network_simulator: NetworkSimulator,
}

struct Internal {
    ip_addr: IpAddr,
    next_tcp_port: u16,
    bound_udp_sockets: HashSet<SocketAddr>,
}

impl HostSimulator {
    pub fn new(network_simulator: NetworkSimulator, ip_addr: IpAddr) -> Self {
        let internal = Internal {
            ip_addr,
            next_tcp_port: 1,
            bound_udp_sockets: HashSet::new(),
        };

        return Self {
            network_simulator,
            internal: Arc::new(Mutex::new(internal)),
        };
    }

    pub fn get_ip_addr(&self) -> IpAddr {
        return self.internal.lock().unwrap().ip_addr;
    }

    pub fn get_network_simulator(&self) -> &NetworkSimulator {
        return &self.network_simulator;
    }

    pub fn connect_tcp(
        &self,
        factory: &SingleThreadedFactory,
        server_socket_addr: SocketAddr,
    ) -> Result<(TcpStream, TcpReader), Error> {
        let port;
        let ip_addr;
        {
            let mut guard = self.internal.lock().unwrap();
            //let mut guard = self.next_tcp_port.lock().unwrap();
            port = guard.next_tcp_port;
            guard.next_tcp_port = guard.next_tcp_port + 1;

            ip_addr = guard.ip_addr;
        }

        let client_socket_addr = SocketAddr::new(ip_addr, port);

        return self
            .network_simulator
            .connect_tcp(factory, client_socket_addr, server_socket_addr);
    }

    pub fn bind_udp_socket(&self, socket_addr: SocketAddr) -> Result<UdpSocketSimulator, Error> {
        let mut guard = self.internal.lock().unwrap();

        if guard.bound_udp_sockets.contains(&socket_addr) {
            return Err(Error::from(ErrorKind::AddrInUse));
        } else {
            guard.bound_udp_sockets.insert(socket_addr.clone());
            return Ok(UdpSocketSimulator::new(self.clone(), socket_addr));
        }
    }

    pub(super) fn drop_udp_socket(&self, socket_addr: &SocketAddr) {
        let mut guard = self.internal.lock().unwrap();

        if !guard.bound_udp_sockets.remove(socket_addr) {
            panic!("Tried to drop a simulated udp socket that was never bound");
        }
    }

    pub(super) fn send_udp(&self, from: &SocketAddr, to: &SocketAddr, buf: &[u8]) {
        self.network_simulator.send_udp(from, to, buf);
    }
}
