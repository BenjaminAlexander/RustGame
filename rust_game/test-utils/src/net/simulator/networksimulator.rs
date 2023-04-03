use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use log::info;
use commons::factory::FactoryTrait;
use commons::net::TcpConnectionHandlerTrait;
use commons::threading::AsyncJoinCallBackTrait;
use commons::threading::channel::{ChannelThreadBuilder, TryRecvError};
use commons::threading::eventhandling::{EventOrStopThread, EventSenderTrait, Sender};
use crate::net::{ChannelTcpReader, ChannelTcpWriter};
use crate::net::simulator::hostsimulator::HostSimulator;
use crate::net::simulator::tcplistenereventhandler::{TcpListenerEvent, TcpListenerEventHandler};
use crate::singlethreaded::{SingleThreadedFactory, TimeQueue};

#[derive(Clone)]
pub struct NetworkSimulator {
    internal: Arc<Mutex<Internal>>,
    time_queue: TimeQueue
}

struct Internal {
    tcp_listeners: HashMap<SocketAddr, Sender<SingleThreadedFactory, TcpListenerEvent>>,
}

impl NetworkSimulator {
    pub fn new(time_queue: TimeQueue) -> Self {

        let internal = Internal {
            tcp_listeners: HashMap::new()
        };

        return Self {
            internal: Arc::new(Mutex::new(internal)),
            time_queue
        }
    }

    pub fn new_host(&self, ip_addr: IpAddr) -> HostSimulator {
        return HostSimulator::new(self.clone(), ip_addr);
    }

    fn new_tcp_channel(factory: &SingleThreadedFactory, src_socket_addr: SocketAddr, dest_socket_addr: SocketAddr) -> (ChannelTcpWriter, ChannelTcpReader) {

        //TODO: make this an EventOrStop so it can be used for an event handler
        //TODO: or, even better, make a channel thread builder and stash it in the reader
        let (sender, receiver) = factory.new_channel::<Vec<u8>>().take();
        let reader = ChannelTcpReader::new(dest_socket_addr, src_socket_addr, receiver);
        let writer = ChannelTcpWriter::new(src_socket_addr, dest_socket_addr, sender);
        return (writer, reader);
    }

    pub fn spawn_tcp_listener<TcpConnectionHandler: TcpConnectionHandlerTrait<Factory=SingleThreadedFactory>>(
        &self,
        factory: &SingleThreadedFactory,
        socket_adder: SocketAddr,
        thread_builder: ChannelThreadBuilder<SingleThreadedFactory, EventOrStopThread<()>>,
        connection_handler: TcpConnectionHandler,
        join_call_back: impl AsyncJoinCallBackTrait<SingleThreadedFactory, TcpConnectionHandler>
    ) -> Result<Sender<SingleThreadedFactory, ()>, Error> {

        let mut guard = self.internal.lock().unwrap();

        if guard.tcp_listeners.contains_key(&socket_adder) {
            return Err(Error::from(ErrorKind::AddrInUse));
        }

        let (thread_builder, channel) = thread_builder.take();

        let tcp_listener_event_handler = TcpListenerEventHandler::new(
            connection_handler,
        );

        let sender = thread_builder.spawn_event_handler(tcp_listener_event_handler, join_call_back).unwrap();

        let (sender_to_return, mut receiver) = channel.take();

        loop {
            match receiver.try_recv() {
                Ok(EventOrStopThread::Event(())) => {}
                Ok(EventOrStopThread::StopThread) => {
                    sender.send_stop_thread().unwrap();
                }
                Err(TryRecvError::Empty) => {
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    sender.send_stop_thread().unwrap();
                }
            }
        }

        let receiver = RefCell::new(receiver);
        let sender_clone = sender.clone();
        sender_to_return.set_on_send(move ||{
            match receiver.borrow_mut().try_recv() {
                Ok(EventOrStopThread::Event(())) => {}
                Ok(EventOrStopThread::StopThread) => {
                    sender_clone.send_stop_thread().unwrap();
                }
                Err(TryRecvError::Empty) => {
                    panic!("on_send was called when there is nothing in the channel")
                }
                Err(TryRecvError::Disconnected) => {
                    sender_clone.send_stop_thread().unwrap();
                }
            }
        });

        guard.tcp_listeners.insert(socket_adder, sender);

        return Ok(sender_to_return);
    }

    pub fn connect_tcp(&self, factory: &SingleThreadedFactory, client_socket_addr: SocketAddr, server_socket_addr: SocketAddr) -> Result<(ChannelTcpWriter, ChannelTcpReader), Error> {

        let mut guard = self.internal.lock().unwrap();

        if let Some(sender) = guard.tcp_listeners.get(&server_socket_addr) {


            let (write_server_to_client, read_server_to_client) = Self::new_tcp_channel(factory, server_socket_addr, client_socket_addr);
            let (write_client_to_server, read_client_to_server) = Self::new_tcp_channel(factory, client_socket_addr, server_socket_addr);

            sender.send_event(TcpListenerEvent::Connection(write_server_to_client, read_client_to_server)).unwrap();

            return Ok((write_client_to_server, read_server_to_client));
        } else {

            info!("{:?} tried to connect (TCP) to {:?} but there is no listener at that SocketAddr.", client_socket_addr, server_socket_addr);
            return Err(Error::from(ErrorKind::ConnectionRefused));
        }
    }
}

