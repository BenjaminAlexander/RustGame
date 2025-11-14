use crate::net::{
    UdpReadHandlerTrait,
    UdpSocketTrait,
};
use crate::real_time::simulation::net::host_simulator::HostSimulator;
use crate::real_time::simulation::net::network_simulator::NetworkSimulator;
use crate::real_time::{EventOrStopThread, Receiver};
use std::io::Error;
use std::net::SocketAddr;
use std::sync::{
    Arc,
    Mutex,
};

#[derive(Clone)]
pub struct UdpSocketSimulator {
    internal: Arc<Mutex<Internal>>,
    network_simulator: NetworkSimulator,
}

struct Internal {
    socket_addr: SocketAddr,
    host_simulator: HostSimulator,
}

impl UdpSocketSimulator {
    pub fn new(host_simulator: HostSimulator, socket_addr: SocketAddr) -> Self {
        let network_simulator = host_simulator.get_network_simulator().clone();

        let internal = Internal {
            socket_addr,
            host_simulator: host_simulator.clone(),
        };

        return Self {
            internal: Arc::new(Mutex::new(internal)),
            network_simulator,
        };
    }

    pub fn local_addr(&self) -> SocketAddr {
        return self.internal.lock().unwrap().socket_addr;
    }

    pub fn spawn_simulated_udp_reader<T: UdpReadHandlerTrait>(
        self,
        thread_name: String,
        receiver: Receiver<EventOrStopThread<()>>,
        udp_read_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> Result<(), Error> {
        return receiver.spawn_simulated_udp_reader(
            self.network_simulator.clone(),
            thread_name,
            self,
            udp_read_handler,
            join_call_back,
        );
    }

    pub fn get_network_simulator(&self) -> &NetworkSimulator {
        return &self.network_simulator;
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
