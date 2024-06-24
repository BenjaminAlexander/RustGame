use crate::net::HostSimulator;
use commons::net::UdpSocketTrait;
use std::io::Error;
use std::net::SocketAddr;
use std::sync::{
    Arc,
    Mutex,
};

#[derive(Clone)]
pub struct UdpSocketSimulator {
    internal: Arc<Mutex<Internal>>,
}

struct Internal {
    socket_addr: SocketAddr,
    host_simulator: HostSimulator,
}

impl UdpSocketSimulator {
    pub fn new(host_simulator: HostSimulator, socket_addr: SocketAddr) -> Self {
        let internal = Internal {
            socket_addr,
            host_simulator,
        };

        return Self {
            internal: Arc::new(Mutex::new(internal)),
        };
    }

    pub fn get_socket_addr(&self) -> SocketAddr {
        return self.internal.lock().unwrap().socket_addr;
    }
}

impl Drop for Internal {
    fn drop(&mut self) {
        self.host_simulator.drop_udp_socket(&self.socket_addr)
    }
}

impl UdpSocketTrait for UdpSocketSimulator {
    fn send_to(&mut self, buf: &[u8], socket_addr: &SocketAddr) -> Result<usize, Error> {
        let guard = self.internal.lock().unwrap();
        guard
            .host_simulator
            .send_udp(&guard.socket_addr, socket_addr, buf);
        return Ok(buf.len());
    }

    fn try_clone(&self) -> Result<Self, Error> {
        return Ok(self.clone());
    }
}
