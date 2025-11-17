use std::{
    io::Error,
    net::SocketAddr,
};

use crate::real_time::{
    EventHandlerStopper, EventOrStopThread, EventSender, FactoryTrait, Receiver, net::tcp::tcp_connection_handler_trait::TcpConnectionHandlerTrait, real::net::tcp::TcpListenerEventHandler, receiver::ReceiverImplementation, simulation::net::NetworkSimulator
};

pub struct TcpListenerBuilder {
    stopper: EventHandlerStopper,
    receiver: Receiver<EventOrStopThread<()>>,
}

impl TcpListenerBuilder {
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

    pub fn spawn_thread_with_call_back<T: TcpConnectionHandlerTrait>(
        self,
        thread_name: String,
        socket_addr: SocketAddr,
        tcp_connection_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> Result<EventHandlerStopper, Error> {
        match self.receiver.take_implementation() {
            ReceiverImplementation::Real(real_receiver) => TcpListenerEventHandler::spawn_tcp_listener(
                thread_name,
                real_receiver,
                socket_addr,
                tcp_connection_handler,
                join_call_back,
            ),
            //TODO: reorder args to match above
            ReceiverImplementation::Simulated(single_threaded_receiver) => NetworkSimulator::spawn_tcp_listener(
                socket_addr,
                thread_name,
                single_threaded_receiver,
                tcp_connection_handler,
                join_call_back,
            ),
        }?;
        return Ok(self.stopper);
    }

    pub fn spawn_thread<T: TcpConnectionHandlerTrait>(
        self,
        thread_name: String,
        socket_addr: SocketAddr,
        tcp_connection_handler: T,
    ) -> Result<EventHandlerStopper, Error> {
        return self.spawn_thread_with_call_back(
            thread_name,
            socket_addr,
            tcp_connection_handler,
            |_| {},
        );
    }

    pub fn new_thread<T: TcpConnectionHandlerTrait>(
        factory: &impl FactoryTrait,
        thread_name: String,
        socket_addr: SocketAddr,
        tcp_connection_handler: T,
    ) -> Result<EventHandlerStopper, Error> {
        return Self::new(factory).spawn_thread(thread_name, socket_addr, tcp_connection_handler);
    }
}
