use crate::real_time::factory::FactoryImplementation;
use crate::real_time::net::tcp::{
    TcpReader,
    TcpStream,
};
use crate::real_time::net::udp::UdpSocket;
use crate::real_time::receiver::ReceiverImplementation;
use crate::real_time::sender::SenderImplementation;
use crate::real_time::simulation::net::{
    HostSimulator,
    NetworkSimulator,
};
use crate::real_time::simulation::single_threaded_receiver::SingleThreadedReceiver;
use crate::real_time::simulation::time_queue::TimeQueue;
use crate::real_time::simulation::SimulatedTimeSource;
use crate::real_time::{
    Factory,
    Receiver,
    Sender,
    TimeSource,
};
use std::io::Error;
use std::net::{
    IpAddr,
    Ipv4Addr,
    SocketAddr,
};

#[derive(Clone)]
pub struct SingleThreadedFactory {
    time_source: TimeSource,
    simulated_time_source: SimulatedTimeSource,
    //TODO: don't let this TimeQueue escape SingleThreaded package
    time_queue: TimeQueue,
    host_simulator: HostSimulator,
}

impl SingleThreadedFactory {
    pub fn new() -> Self {
        let (time_source, simulated_time_source) = TimeSource::new_simulated_time_source();
        let time_queue = TimeQueue::new(simulated_time_source.clone());

        return Self {
            time_source,
            simulated_time_source,
            host_simulator: NetworkSimulator::new()
                .new_host(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            time_queue,
        };
    }

    pub fn get_simulated_time_source(&self) -> &SimulatedTimeSource {
        return &self.simulated_time_source;
    }

    pub fn get_time_queue(&self) -> &TimeQueue {
        return &self.time_queue;
    }

    pub fn get_host_simulator(&self) -> &HostSimulator {
        return &self.host_simulator;
    }

    pub fn clone_for_new_host(&self, ip_adder: IpAddr) -> Self {
        let mut clone = self.clone();
        clone.host_simulator = clone
            .host_simulator
            .get_network_simulator()
            .new_host(ip_adder);
        return clone;
    }

    pub fn get_time_source(&self) -> &TimeSource {
        return &self.time_source;
    }

    pub fn new_channel<T: Send>(&self) -> (Sender<T>, Receiver<T>) {
        let (simulated_sender, simulated_receiver) = SingleThreadedReceiver::new(self.clone());
        let sender = Sender::new(SenderImplementation::Simulated(simulated_sender));
        let receiver = Receiver::new(ReceiverImplementation::Simulated(simulated_receiver));
        return (sender, receiver);
    }

    pub fn connect_tcp(&self, socket_addr: SocketAddr) -> Result<(TcpStream, TcpReader), Error> {
        return self.host_simulator.connect_tcp(&self, socket_addr);
    }

    pub fn bind_udp_socket(&self, socket_addr: SocketAddr) -> Result<UdpSocket, Error> {
        return Ok(UdpSocket::new_simulated(
            self.host_simulator.bind_udp_socket(socket_addr)?,
        ));
    }
}

impl Into<Factory> for SingleThreadedFactory {
    fn into(self) -> Factory {
        Factory {
            implementation: FactoryImplementation::Simulated(self),
        }
    }
}
