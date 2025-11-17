use crate::real_time::net::udp::UdpReadHandlerTrait;
use crate::real_time::simulation::net::udp::UdpSocketSimulator;
use crate::real_time::simulation::net::NetworkSimulator;
use crate::real_time::simulation::receiver_link::{
    ReceiveOrDisconnected,
    ReceiverLink,
};
use crate::real_time::simulation::sender_link::SenderLink;
use crate::real_time::simulation::single_threaded_sender::SingleThreadedSender;
use crate::real_time::simulation::SingleThreadedFactory;
use crate::real_time::{
    EventOrStopThread,
    FactoryTrait,
    ReceiveMetaData,
};
use std::io::Error;
use std::sync::mpsc::TryRecvError;

pub struct SingleThreadedReceiver<T: Send> {
    factory: SingleThreadedFactory,
    link: ReceiverLink<T>,
}

impl<T: Send> SingleThreadedReceiver<T> {
    pub fn new(factory: SingleThreadedFactory) -> (SingleThreadedSender<T>, Self) {
        let receiver_link = ReceiverLink::new(factory.get_time_source().clone());
        let sender_link = SenderLink::new(receiver_link.clone());
        let sender = SingleThreadedSender::new(sender_link);
        let receiver = Self {
            factory,
            link: receiver_link,
        };

        return (sender, receiver);
    }

    pub fn get_factory(&self) -> &SingleThreadedFactory {
        return &self.factory;
    }

    pub fn try_recv_meta_data(&mut self) -> Result<(ReceiveMetaData, T), TryRecvError> {
        return self.link.try_recv_meta_data();
    }

    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        let (_, value) = self.try_recv_meta_data()?;
        return Ok(value);
    }

    pub fn to_consumer(
        self,
        consumer: impl Fn(ReceiveOrDisconnected<T>) -> Result<(), T> + Send + 'static,
    ) -> ReceiverLink<T> {
        self.link.to_consumer(consumer);
        return self.link;
    }
}

impl SingleThreadedReceiver<EventOrStopThread<()>> {

    pub fn spawn_simulated_udp_reader<T: UdpReadHandlerTrait>(
        self,
        network_simulator: NetworkSimulator,
        thread_name: String,
        udp_socket_simulator: UdpSocketSimulator,
        udp_read_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> Result<(), Error> {
        return network_simulator.spawn_udp_reader(
            self.factory.clone(),
            thread_name,
            self,
            udp_socket_simulator,
            udp_read_handler,
            join_call_back,
        );
    }
}
