use std::{
    io::Error,
    net::SocketAddr,
    sync::mpsc::TryRecvError,
};

use crate::{
    net::{
        RealTcpStream,
        RealUdpSocket,
        TcpConnectionHandlerTrait,
        TcpReadHandlerTrait,
        UdpReadHandlerTrait,
    },
    single_threaded_simulator::{
        SingleThreadedFactory, SingleThreadedReceiver, SingleThreadedSender, net::{
            NetworkSimulator,
            UdpSocketSimulator,
        }
    },
    threading::{
        channel::{
            RealReceiver,
            RealSender,
            ReceiveMetaData,
            ReceiverTrait,
            SenderTrait,
        },
        eventhandling::{
            EventHandlerTrait, EventOrStopThread, spawn_event_handler
        },
    },
};

//TODO: don't expose new functions
pub(crate) fn new_simulated_channel<T: Send>(factory: &SingleThreadedFactory) -> (Sender<T>, Receiver<T>) {
    let (simulated_sender, simulated_receiver) = SingleThreadedReceiver::new(factory.clone());
    let sender = Sender::new(SenderImplementation::Simulated(simulated_sender));
    let receiver = Receiver::new(ReceiverImplementation::Simulated(simulated_receiver));
    return (sender, receiver);
}

//TODO: hide
pub enum SenderImplementation<T: Send> {
    Real(RealSender<T>),

    //TODO: conditionally compile
    Simulated(SingleThreadedSender<T>),
}

impl<T: Send> Clone for SenderImplementation<T> {
    fn clone(&self) -> Self {
        match &self {
            SenderImplementation::Real(real_sender) => {
                SenderImplementation::Real(real_sender.clone())
            }
            SenderImplementation::Simulated(simulated_sender) => {
                SenderImplementation::Simulated(simulated_sender.clone())
            }
        }
    }
}

pub struct Sender<T: Send> {
    implementation: SenderImplementation<T>,
}

impl<T: Send> Sender<T> {
    //TODO: hide
    pub fn new(implementation: SenderImplementation<T>) -> Self {
        return Self { implementation };
    }

    pub fn send(&self, value: T) -> Result<(), T> {
        match &self.implementation {
            SenderImplementation::Real(real_sender) => real_sender.send(value),
            SenderImplementation::Simulated(simulated_sender) => simulated_sender.send(value),
        }
    }
}

impl<T: Send> Clone for Sender<T> {
    fn clone(&self) -> Self {
        return Self {
            implementation: self.implementation.clone(),
        };
    }
}

impl<T: Send> Sender<EventOrStopThread<T>> {
    pub fn send_event(&self, event: T) -> Result<(), T> {
        return match self.send(EventOrStopThread::Event(event)) {
            Ok(_) => Ok(()),
            Err(EventOrStopThread::Event(event)) => Err(event),
            _ => panic!("Unreachable"),
        };
    }

    pub fn send_stop_thread(&self) -> Result<(), ()> {
        return match self.send(EventOrStopThread::StopThread) {
            Ok(_) => Ok(()),
            Err(EventOrStopThread::StopThread) => Err(()),
            _ => panic!("Unreachable"),
        };
    }
}

//TODO: hide
pub enum ReceiverImplementation<T: Send> {
    Real(RealReceiver<T>),

    //TODO: conditionally compile
    Simulated(SingleThreadedReceiver<T>),
}

pub struct Receiver<T: Send> {
    implementation: ReceiverImplementation<T>,
}

impl<T: Send> Receiver<T> {
    //TODO: hide
    pub fn new(implementation: ReceiverImplementation<T>) -> Self {
        return Self { implementation };
    }
}

impl<T: Send> ReceiverTrait<T> for Receiver<T> {
    fn try_recv_meta_data(&mut self) -> Result<(ReceiveMetaData, T), TryRecvError> {
        match &mut self.implementation {
            ReceiverImplementation::Real(real_receiver) => real_receiver.try_recv_meta_data(),
            ReceiverImplementation::Simulated(simulated_receiver) => {
                simulated_receiver.try_recv_meta_data()
            }
        }
    }
}

impl<T: Send> Receiver<EventOrStopThread<T>> {
    pub fn spawn_event_handler<U: EventHandlerTrait<Event = T>>(
        self,
        thread_name: String,
        event_handler: U,
        join_call_back: impl FnOnce(U::ThreadReturn) + Send + 'static,
    ) -> std::io::Result<()> {
        match self.implementation {
            ReceiverImplementation::Real(real_receiver) => {
                spawn_event_handler(thread_name, real_receiver, event_handler, join_call_back)
            }
            ReceiverImplementation::Simulated(single_threaded_receiver) => single_threaded_receiver
                .spawn_event_handler(thread_name, event_handler, join_call_back),
        }
    }
}

impl Receiver<EventOrStopThread<()>> {
    pub fn spawn_tcp_listener<T: TcpConnectionHandlerTrait>(
        self,
        thread_name: String,
        socket_addr: SocketAddr,
        tcp_connection_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> Result<(), Error> {
        match self.implementation {
            ReceiverImplementation::Real(real_receiver) => real_receiver.spawn_tcp_listener(
                thread_name,
                socket_addr,
                tcp_connection_handler,
                join_call_back,
            ),
            ReceiverImplementation::Simulated(single_threaded_receiver) => single_threaded_receiver
                .spawn_tcp_listener(
                    thread_name,
                    socket_addr,
                    tcp_connection_handler,
                    join_call_back,
                ),
        }
    }

    pub fn spawn_real_tcp_reader<T: TcpReadHandlerTrait>(
        self,
        thread_name: String,
        real_tcp_stream: RealTcpStream,
        tcp_read_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> Result<(), Error> {
        match self.implementation {
            ReceiverImplementation::Real(real_receiver) => real_receiver.spawn_real_tcp_reader(thread_name, real_tcp_stream, tcp_read_handler, join_call_back),
            ReceiverImplementation::Simulated(_) => panic!("Spawning a TCP reader thread with a real TCP stream and a simulated channel isn't implemented"),
        }
    }

    pub fn spawn_simulated_tcp_reader<T: TcpReadHandlerTrait>(
        self,
        thread_name: String,
        simulated_tcp_reader: SingleThreadedReceiver<Vec<u8>>,
        tcp_read_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> Result<(), Error> {
        match self.implementation {
            ReceiverImplementation::Real(_) => panic!("Spawning a TCP reader thread with a simulated TCP reader and a real channel isn't implemented"),
            ReceiverImplementation::Simulated(single_threaded_receiver) => single_threaded_receiver.spawn_simulated_tcp_reader(thread_name, simulated_tcp_reader, tcp_read_handler, join_call_back),
        }
    }

    pub fn spawn_real_udp_reader<T: UdpReadHandlerTrait>(
        self,
        thread_name: String,
        real_udp_socket: RealUdpSocket,
        udp_read_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> Result<(), Error> {
        match self.implementation {
            ReceiverImplementation::Real(real_receiver) => real_receiver.spawn_real_udp_reader(thread_name, real_udp_socket, udp_read_handler, join_call_back),
            ReceiverImplementation::Simulated(_) => panic!("Spawning a UDP reader thread with a real UDP socket and a simulated channel isn't implemented"),
        }
    }

    pub fn spawn_simulated_udp_reader<T: UdpReadHandlerTrait>(
        self,
        network_simulator: NetworkSimulator,
        thread_name: String,
        udp_socket_simulator: UdpSocketSimulator,
        udp_read_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> Result<(), Error> {
        match self.implementation {
            ReceiverImplementation::Real(_) => panic!("Spawning a UDP reader thread with a simulated UDP socket and a real channel isn't implemented"),
            ReceiverImplementation::Simulated(single_threaded_receiver) => single_threaded_receiver.spawn_simulated_udp_reader(network_simulator, thread_name, udp_socket_simulator, udp_read_handler, join_call_back),
        }
    }
}
