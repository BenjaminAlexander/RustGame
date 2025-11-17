use crate::real_time::simulation::net::host_simulator::HostSimulator;
use std::io::Error;
use std::net::SocketAddr;
use std::sync::{
    Arc,
    Mutex,
};

#[derive(Clone)]
pub struct UdpSocketSimulator {
    internal: Arc<Mutex<Internal>>
}

struct Internal {
    socket_addr: SocketAddr,
    host_simulator: HostSimulator,
}

impl UdpSocketSimulator {
    pub fn new(host_simulator: HostSimulator, socket_addr: SocketAddr) -> Self {
        let internal = Internal {
            socket_addr,
            host_simulator: host_simulator.clone(),
        };

        return Self {
            internal: Arc::new(Mutex::new(internal)),
        };
    }

    pub fn local_addr(&self) -> SocketAddr {
        return self.internal.lock().unwrap().socket_addr;
    }

    pub fn send_to(&mut self, buf: &[u8], socket_addr: &SocketAddr) -> Result<usize, Error> {
        let guard = self.internal.lock().unwrap();
        guard
            .host_simulator
            .send_udp(&guard.socket_addr, socket_addr, buf);
        return Ok(buf.len());
    }

    pub fn try_clone(&self) -> Result<Self, Error> {
        return Ok(self.clone());
    }
}

impl Drop for Internal {
    fn drop(&mut self) {
        self.host_simulator.drop_udp_socket(&self.socket_addr)
    }
}
