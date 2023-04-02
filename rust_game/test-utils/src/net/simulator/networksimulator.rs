use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::marker::PhantomData;
use std::net::{IpAddr, SocketAddr};
use std::ops::ControlFlow;
use std::sync::{Arc, Mutex};
use log::info;
use commons::factory::FactoryTrait;
use commons::net::TcpConnectionHandlerTrait;
use commons::threading::{AsyncJoinCallBackTrait, channel, ThreadBuilder};
use commons::threading::channel::Receiver;
use commons::threading::eventhandling::{EventOrStopThread, Sender};
use crate::net::{ChannelTcpReader, ChannelTcpWriter};
use crate::net::simulator::hostsimulator::HostSimulator;
use crate::net::simulator::tcpconnectionhandlerholdertrait;
use crate::net::simulator::tcpconnectionhandlerholdertrait::TcpConnectionHandlerHolderTrait;
use crate::singlethreaded::{SingleThreadedFactory, SingleThreadedSender, TimeQueue};

#[derive(Clone)]
pub struct NetworkSimulator {
    internal: Arc<Mutex<Internal>>,
    time_queue: TimeQueue
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

    fn insert_tcp_listener(&self, socket_adder: SocketAddr, tcp_connection_handler_holder: Box<dyn TcpConnectionHandlerHolderTrait>) {
        self.internal.lock().unwrap().insert_tcp_listener(socket_adder, tcp_connection_handler_holder);
    }

    fn remove_tcp_listener(&self, socket_adder: &SocketAddr) -> Option<Box<dyn TcpConnectionHandlerHolderTrait>> {
        return self.internal.lock().unwrap().remove_tcp_listener(socket_adder);
    }

    fn contains_tcp_listener(&self, socket_adder: &SocketAddr) -> bool {
        return self.internal.lock().unwrap().contains_tcp_listener(socket_adder);
    }

    fn new_tcp_channel(factory: &SingleThreadedFactory, src_socket_addr: SocketAddr, dest_socket_addr: SocketAddr) -> (ChannelTcpWriter<SingleThreadedFactory>, ChannelTcpReader<SingleThreadedFactory>) {

        let (sender, receiver) = factory.new_channel::<Vec<u8>>().take();
        let reader = ChannelTcpReader::new(dest_socket_addr, src_socket_addr, receiver);
        let writer = ChannelTcpWriter::new(src_socket_addr, dest_socket_addr, sender);
        return (writer, reader);
    }

    pub fn spawn_tcp_listener<TcpConnectionHandler: TcpConnectionHandlerTrait<TcpSender=ChannelTcpWriter<SingleThreadedFactory>, TcpReceiver=ChannelTcpReader<SingleThreadedFactory>>>(
        &self,
        socket_adder: SocketAddr,
        thread_builder: channel::ThreadBuilder<SingleThreadedFactory, EventOrStopThread<()>>,
        connection_handler: TcpConnectionHandler,
        join_call_back: impl AsyncJoinCallBackTrait<SingleThreadedFactory, TcpConnectionHandler>
    ) -> Sender<SingleThreadedFactory, ()> {

        let (thread_builder, channel) = thread_builder.take();
        let (sender, receiver) = channel.take();

        let tcp_connection_handler_holder = tcpconnectionhandlerholdertrait::new(
            thread_builder,
            receiver,
            connection_handler,
            join_call_back
        );

        self.insert_tcp_listener(socket_adder, tcp_connection_handler_holder);


        let self_clone = self.clone();
        sender.set_on_send(move ||{
            let self_clone_clone = self_clone.clone();
            self_clone.time_queue.add_event_now(move ||{

                let removed = self_clone_clone.remove_tcp_listener(&socket_adder);
                if let Some(mut tcp_connection_handler_holder) = removed {
                    if let Some(tcp_connection_handler_holder) = tcp_connection_handler_holder.on_send() {
                        self_clone_clone.insert_tcp_listener(socket_adder, tcp_connection_handler_holder);
                    }
                }
            });
        });

        return sender;
    }

    pub fn connect_tcp(&self, factory: &SingleThreadedFactory, client_socket_addr: SocketAddr, server_socket_addr: SocketAddr) -> Result<(ChannelTcpWriter<SingleThreadedFactory>, ChannelTcpReader<SingleThreadedFactory>), Error> {

        if self.contains_tcp_listener(&server_socket_addr) {

            let (write_server_to_client, read_server_to_client) = Self::new_tcp_channel(factory, server_socket_addr, client_socket_addr);
            let (write_client_to_server, read_client_to_server) = Self::new_tcp_channel(factory, client_socket_addr, server_socket_addr);

            let self_clone = self.clone();
            self.time_queue.add_event_now(move ||{
                if let Some(holder) = self_clone.remove_tcp_listener(&server_socket_addr) {
                    if let Some(holder) = holder.on_connection(write_server_to_client, read_client_to_server) {
                        self_clone.insert_tcp_listener(server_socket_addr, holder);
                    }
                }
            });

            return Ok((write_client_to_server, read_server_to_client));
        }

        info!("{:?} tried to connect (TCP) to {:?} but there is no listener at that SocketAddr.", client_socket_addr, server_socket_addr);
        return Err(Error::from(ErrorKind::ConnectionRefused));
    }
}

struct Internal {
    tcp_listeners: HashMap<SocketAddr, Box<dyn TcpConnectionHandlerHolderTrait>>,
}

impl Internal {

    fn insert_tcp_listener(&mut self, socket_adder: SocketAddr, tcp_connection_handler_holder: Box<dyn TcpConnectionHandlerHolderTrait>) {
        //TODO: check if SocketAddr is already in use
        self.tcp_listeners.insert(socket_adder, tcp_connection_handler_holder);
    }

    fn remove_tcp_listener(&mut self, socket_adder: &SocketAddr) -> Option<Box<dyn TcpConnectionHandlerHolderTrait>> {
        return self.tcp_listeners.remove(socket_adder);
    }

    fn contains_tcp_listener(&mut self, socket_adder: &SocketAddr) -> bool {
        return self.tcp_listeners.contains_key(socket_adder);
    }
}

