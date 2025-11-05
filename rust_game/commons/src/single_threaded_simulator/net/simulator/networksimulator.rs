use crate::factory::FactoryTrait;
use crate::net::{
    TcpConnectionHandlerTrait,
    TcpStream,
    UdpReadHandlerTrait,
};
use crate::single_threaded_simulator::net::simulator::hostsimulator::HostSimulator;
use crate::single_threaded_simulator::net::simulator::tcplistenereventhandler::{
    TcpListenerEvent,
    TcpListenerEventHandler,
};
use crate::single_threaded_simulator::net::simulator::udpreadeventhandler::UdpReadEventHandler;
use crate::single_threaded_simulator::net::{
    ChannelTcpWriter,
    UdpSocketSimulator,
};
use crate::single_threaded_simulator::{
    ReceiveOrDisconnected,
    SingleThreadedFactory,
    SingleThreadedReceiver,
};
use crate::threading::channel::{
    ChannelThreadBuilder,
    Sender,
};
use crate::threading::eventhandling::{
    EventHandlerSender,
    EventOrStopThread,
};
use crate::threading::AsyncJoinCallBackTrait;
use log::{
    info,
    warn,
};
use std::collections::HashMap;
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
pub struct NetworkSimulator {
    internal: Arc<Mutex<Internal>>,
}

struct Internal {
    //TODO: add a way to remove TCP listeners when they stop listening
    tcp_listeners: HashMap<SocketAddr, EventHandlerSender<TcpListenerEvent>>,
    udp_readers: HashMap<SocketAddr, EventHandlerSender<(SocketAddr, Vec<u8>)>>,
}

impl NetworkSimulator {
    pub fn new() -> Self {
        let internal = Internal {
            tcp_listeners: HashMap::new(),
            udp_readers: HashMap::new(),
        };

        return Self {
            internal: Arc::new(Mutex::new(internal)),
        };
    }

    pub fn new_host(&self, ip_addr: IpAddr) -> HostSimulator {
        return HostSimulator::new(self.clone(), ip_addr);
    }

    fn new_tcp_channel(
        factory: &SingleThreadedFactory,
        dest_socket_addr: SocketAddr,
    ) -> (ChannelTcpWriter, SingleThreadedReceiver<Vec<u8>>) {
        //TODO: make this an EventOrStop so it can be used for an event handler
        //TODO: or, even better, make a channel thread builder and stash it in the reader
        let (sender, reader) = factory.new_channel::<Vec<u8>>().take();
        let writer = ChannelTcpWriter::new(dest_socket_addr, sender);
        return (writer, reader);
    }

    pub fn spawn_tcp_listener<
        TcpConnectionHandler: TcpConnectionHandlerTrait<SingleThreadedFactory>,
    >(
        &self,
        factory: SingleThreadedFactory,
        socket_addr: SocketAddr,
        thread_builder: ChannelThreadBuilder<SingleThreadedFactory, EventOrStopThread<()>>,
        connection_handler: TcpConnectionHandler,
        join_call_back: impl AsyncJoinCallBackTrait<TcpConnectionHandler>,
    ) -> Result<EventHandlerSender<()>, Error> {
        let mut guard = self.internal.lock().unwrap();

        if guard.tcp_listeners.contains_key(&socket_addr) {
            return Err(Error::from(ErrorKind::AddrInUse));
        }

        let (thread_builder, channel) = thread_builder.take();

        let tcp_listener_event_handler =
            TcpListenerEventHandler::new(socket_addr, connection_handler);

        let sender = thread_builder
            .spawn_event_handler(factory, tcp_listener_event_handler, join_call_back)
            .unwrap();

        let (sender_to_return, receiver) = channel.take();

        let sender_clone = sender.clone();
        receiver.to_consumer(move |receive_or_disconnect| {
            let result = match receive_or_disconnect {
                ReceiveOrDisconnected::Receive(_, EventOrStopThread::Event(())) => Ok(()),
                ReceiveOrDisconnected::Receive(_, EventOrStopThread::StopThread) => {
                    sender_clone.send_stop_thread()
                }
                ReceiveOrDisconnected::Disconnected => sender_clone.send_stop_thread(),
            };

            return match result {
                Ok(()) => Ok(()),
                Err(_) => Err(EventOrStopThread::StopThread),
            };
        });

        guard.tcp_listeners.insert(socket_addr, sender.clone());

        let send_result = sender.send_event(TcpListenerEvent::ListenerReady);

        if send_result.is_err() {
            warn!("Failed to send ListenerReady");
            return Err(Error::from(ErrorKind::BrokenPipe));
        }

        return Ok(sender_to_return);
    }

    pub fn connect_tcp(
        &self,
        factory: &SingleThreadedFactory,
        client_socket_addr: SocketAddr,
        server_socket_addr: SocketAddr,
    ) -> Result<(TcpStream, SingleThreadedReceiver<Vec<u8>>), Error> {
        let guard = self.internal.lock().unwrap();

        if let Some(sender) = guard.tcp_listeners.get(&server_socket_addr) {
            let (write_server_to_client, read_server_to_client) =
                Self::new_tcp_channel(factory, client_socket_addr);
            let (write_client_to_server, read_client_to_server) =
                Self::new_tcp_channel(factory, server_socket_addr);

            let send_result = sender.send_event(TcpListenerEvent::Connection(
                write_server_to_client,
                read_client_to_server,
            ));

            if send_result.is_err() {
                panic!("Failed to send event");
            }

            return Ok((
                TcpStream::new_simulated(write_client_to_server),
                read_server_to_client,
            ));
        } else {
            info!(
                "{:?} tried to connect (TCP) to {:?} but there is no listener at that SocketAddr.",
                client_socket_addr, server_socket_addr
            );
            return Err(Error::from(ErrorKind::ConnectionRefused));
        }
    }

    pub fn spawn_udp_reader<T: UdpReadHandlerTrait>(
        &self,
        factory: SingleThreadedFactory,
        thread_builder: ChannelThreadBuilder<SingleThreadedFactory, EventOrStopThread<()>>,
        udp_socket: UdpSocketSimulator,
        udp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T>,
    ) -> Result<EventHandlerSender<()>, Error> {
        let mut guard = self.internal.lock().unwrap();

        let socket_addr = udp_socket.get_socket_addr();

        if guard.udp_readers.contains_key(&socket_addr) {
            return Err(Error::from(ErrorKind::AddrInUse));
        }

        let (thread_builder, channel) = thread_builder.take();

        let udp_read_event_handler =
            UdpReadEventHandler::new(self.clone(), udp_socket.get_socket_addr(), udp_read_handler);

        let sender = thread_builder
            .spawn_event_handler(factory, udp_read_event_handler, join_call_back)
            .unwrap();

        let (sender_to_return, receiver) = channel.take();

        let sender_clone = sender.clone();
        receiver.to_consumer(move |receive_or_disconnect| {
            let result = match receive_or_disconnect {
                ReceiveOrDisconnected::Receive(_, EventOrStopThread::Event(())) => Ok(()),
                ReceiveOrDisconnected::Receive(_, EventOrStopThread::StopThread) => {
                    sender_clone.send_stop_thread()
                }
                ReceiveOrDisconnected::Disconnected => sender_clone.send_stop_thread(),
            };

            return match result {
                Ok(()) => Ok(()),
                Err(_) => Err(EventOrStopThread::StopThread),
            };
        });

        guard.udp_readers.insert(socket_addr, sender);

        return Ok(sender_to_return);
    }

    pub(super) fn send_udp(&self, from: &SocketAddr, to: &SocketAddr, buf: &[u8]) {
        let guard = self.internal.lock().unwrap();

        if let Some(sender) = guard.udp_readers.get(to) {
            let buf = Vec::from(buf);

            let send_result = sender.send_event((from.clone(), buf));

            if send_result.is_err() {
                panic!("Failed to send event");
            }
        }
    }

    pub(super) fn remove_udp_reader(
        &self,
        socket_addr: &SocketAddr,
    ) -> Option<Sender<EventOrStopThread<(SocketAddr, Vec<u8>)>>> {
        return self
            .internal
            .lock()
            .unwrap()
            .udp_readers
            .remove(socket_addr);
    }
}
