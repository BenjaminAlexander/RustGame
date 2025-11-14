use crate::real_time::{EventHandlerBuilder, EventOrStopThread, FactoryTrait, ReceiveMetaData, Receiver};
use crate::net::{
    TcpConnectionHandlerTrait,
    TcpReadHandlerTrait,
    UdpReadHandlerTrait,
};
use crate::real_time::simulation::SingleThreadedFactory;
use crate::single_threaded_simulator::channel::receiverlink::{
    ReceiveOrDisconnected,
    ReceiverLink,
};
use crate::single_threaded_simulator::channel::senderlink::SenderLink;
use crate::single_threaded_simulator::net::{
    NetworkSimulator,
    TcpReaderEventHandler,
    UdpSocketSimulator,
};
use crate::single_threaded_simulator::{
    SingleThreadedSender,
};
use std::io::Error;
use std::net::SocketAddr;
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
    pub fn spawn_tcp_listener<T: TcpConnectionHandlerTrait>(
        self,
        thread_name: String,
        socket_addr: SocketAddr,
        tcp_connection_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> std::io::Result<()> {
        return self
            .factory
            .clone()
            .get_host_simulator()
            .get_network_simulator()
            .spawn_tcp_listener(
                self.factory.clone(),
                socket_addr,
                thread_name,
                self,
                tcp_connection_handler,
                join_call_back,
            );
    }

    pub fn spawn_simulated_tcp_reader<T: TcpReadHandlerTrait>(
        self,
        thread_name: String,
        simulated_tcp_reader: SingleThreadedReceiver<Vec<u8>>,
        tcp_read_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> Result<(), Error> {
        let tcp_reader_event_handler = TcpReaderEventHandler::new(tcp_read_handler);

        let sender = EventHandlerBuilder::new(&self.factory)
            .spawn_thread_with_callback(thread_name, tcp_reader_event_handler, join_call_back)
            .unwrap();

        let sender_clone = sender.clone();
        simulated_tcp_reader.to_consumer(move |receive_or_disconnect| {
            match receive_or_disconnect {
                ReceiveOrDisconnected::Receive(_, buf) => {
                    return match sender_clone.send_event(buf) {
                        Ok(_) => Ok(()),
                        Err(buf) => Err(buf),
                    };
                }
                ReceiveOrDisconnected::Disconnected => {
                    let _ = sender_clone.send_stop_thread();
                    return Ok(());
                }
            };
        });

        self.to_consumer(move |receive_or_disconnect| {
            let result = match receive_or_disconnect {
                ReceiveOrDisconnected::Receive(_, EventOrStopThread::Event(())) => Ok(()),
                ReceiveOrDisconnected::Receive(_, EventOrStopThread::StopThread) => {
                    sender.send_stop_thread()
                }
                ReceiveOrDisconnected::Disconnected => sender.send_stop_thread(),
            };

            return match result {
                Ok(()) => Ok(()),
                Err(_) => Err(EventOrStopThread::StopThread),
            };
        });

        return Ok(());
    }

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

impl SingleThreadedReceiver<Vec<u8>> {
    pub fn spawn_simulated_tcp_reader<T: TcpReadHandlerTrait>(
        self,
        thread_name: String,
        receiver: Receiver<EventOrStopThread<()>>,
        tcp_read_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> Result<(), Error> {
        return receiver.spawn_simulated_tcp_reader(
            thread_name,
            self,
            tcp_read_handler,
            join_call_back,
        );
    }
}
