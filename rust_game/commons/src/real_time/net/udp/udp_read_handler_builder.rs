use std::io::Error;

use crate::real_time::{
    net::udp::{
        udp_read_handler_trait::UdpReadHandlerTrait,
        udp_socket::{
            UdpSocket,
            UdpSocketImplementation,
        },
    },
    real::net::udp::RealUdpReaderEventHandler,
    receiver::ReceiverImplementation,
    simulation::net::NetworkSimulator,
    EventHandlerStopper,
    EventOrStopThread,
    EventSender,
    FactoryTrait,
    Receiver,
};

pub struct UdpReadHandlerBuilder {
    stopper: EventHandlerStopper,
    receiver: Receiver<EventOrStopThread<()>>,
}

impl UdpReadHandlerBuilder {
    pub fn new(factory: &impl FactoryTrait) -> Self {
        let (sender, receiver) = factory.new_channel();

        return Self {
            stopper: EventHandlerStopper::new(EventSender::new(sender)),
            receiver,
        };
    }

    pub fn get_stopper(&self) -> &EventHandlerStopper {
        return &self.stopper;
    }

    pub fn spawn_thread_with_call_back<T: UdpReadHandlerTrait>(
        self,
        thread_name: String,
        udp_socket: UdpSocket,
        udp_read_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> Result<EventHandlerStopper, Error> {
        match (self.receiver.take_implementation(), udp_socket.take_implementation()) {
            (ReceiverImplementation::Real(real_receiver), UdpSocketImplementation::Real(real_udp_socket)) => RealUdpReaderEventHandler::spawn_udp_reader(thread_name, real_receiver, real_udp_socket, udp_read_handler, join_call_back),
            (ReceiverImplementation::Real(_), UdpSocketImplementation::Simulated(_)) => panic!("Spawning a UDP reader thread with a simulated UDP socket and a real channel isn't supported"),
            (ReceiverImplementation::Simulated(_), UdpSocketImplementation::Real(_)) => panic!("Spawning a UDP reader thread with a real UDP socket and a simulated channel isn't supported"),
            (ReceiverImplementation::Simulated(single_threaded_receiver), UdpSocketImplementation::Simulated(udp_socket_simulator)) => NetworkSimulator::spawn_udp_reader(thread_name, single_threaded_receiver, udp_socket_simulator, udp_read_handler, join_call_back),
        }?;

        return Ok(self.stopper);
    }

    pub fn spawn_thread<T: UdpReadHandlerTrait>(
        self,
        thread_name: String,
        udp_socket: UdpSocket,
        udp_read_handler: T,
    ) -> Result<EventHandlerStopper, Error> {
        return self.spawn_thread_with_call_back(thread_name, udp_socket, udp_read_handler, |_| {});
    }

    pub fn new_thread<T: UdpReadHandlerTrait>(
        factory: &impl FactoryTrait,
        thread_name: String,
        udp_socket: UdpSocket,
        udp_read_handler: T,
    ) -> Result<EventHandlerStopper, Error> {
        return Self::new(factory).spawn_thread(thread_name, udp_socket, udp_read_handler);
    }
}
