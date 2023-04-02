use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::marker::PhantomData;
use std::net::{IpAddr, SocketAddr};
use std::ops::ControlFlow;
use std::sync::{Arc, Mutex};
use log::info;
use commons::factory::FactoryTrait;
use commons::net::TcpConnectionHandlerTrait;
use commons::threading::{AsyncJoinCallBackTrait, ThreadBuilder};
use commons::threading::channel::Receiver;
use commons::threading::eventhandling::EventOrStopThread;
use crate::net::{ChannelTcpReader, ChannelTcpWriter};
use crate::net::simulator::hostsimulator::HostSimulator;
use crate::net::simulator::tcpconnectionhandlerholdertrait;
use crate::net::simulator::tcpconnectionhandlerholdertrait::TcpConnectionHandlerHolderTrait;
use crate::singlethreaded::TimeQueue;

#[derive(Clone)]
pub struct NetworkSimulator<Factory: FactoryTrait<TcpWriter=ChannelTcpWriter<Factory>, TcpReader=ChannelTcpReader<Factory>>> {
    internal: Arc<Mutex<Internal<Factory>>>,
    time_queue: TimeQueue
}

impl<Factory: FactoryTrait<TcpWriter=ChannelTcpWriter<Factory>, TcpReader=ChannelTcpReader<Factory>>> NetworkSimulator<Factory> {
    pub fn new(time_queue: TimeQueue) -> Self {

        let internal = Internal {
            tcp_listeners: HashMap::new(),
            phantom: PhantomData::default()
        };

        return Self {
            internal: Arc::new(Mutex::new(internal)),
            time_queue
        }
    }

    pub fn new_host(&self, ip_addr: IpAddr) -> HostSimulator<Factory> {
        return HostSimulator::new(self.clone(), ip_addr);
    }

    pub fn start_listener<TcpConnectionHandler: TcpConnectionHandlerTrait<TcpSender=Factory::TcpWriter, TcpReceiver=Factory::TcpReader>>(
        &self,
        socket_adder: SocketAddr,
        thread_builder: ThreadBuilder<Factory>,
        receiver: Receiver<Factory, EventOrStopThread<()>>,
        connection_handler: TcpConnectionHandler,
        join_call_back: impl AsyncJoinCallBackTrait<Factory, TcpConnectionHandler>
    ) {

        let tcp_connection_handler_holder = tcpconnectionhandlerholdertrait::new(
            thread_builder,
            receiver,
            connection_handler,
            join_call_back
        );

        self.internal.lock().unwrap().start_listener(socket_adder, tcp_connection_handler_holder);
    }

    pub fn on_send_to_tcp_listener(&self, socket_adder: SocketAddr) {
        let self_clone = self.clone();
        self.time_queue.add_event_now(move ||{
            self_clone.internal.lock().unwrap().on_send_to_tcp_listener(socket_adder);
        });
    }
}

struct Internal<Factory: FactoryTrait<TcpWriter=ChannelTcpWriter<Factory>, TcpReader=ChannelTcpReader<Factory>>> {
    tcp_listeners: HashMap<SocketAddr, Box<dyn TcpConnectionHandlerHolderTrait<Factory=Factory>>>,
    phantom: PhantomData<Factory>
}

impl<Factory: FactoryTrait<TcpWriter=ChannelTcpWriter<Factory>, TcpReader=ChannelTcpReader<Factory>>> Internal<Factory> {

    fn start_listener(&mut self, socket_adder: SocketAddr, tcp_connection_handler_holder: Box<dyn TcpConnectionHandlerHolderTrait<Factory=Factory>>) {
        //TODO: check if SocketAddr is already in use
        self.tcp_listeners.insert(socket_adder, tcp_connection_handler_holder);
    }

    //TODO: is this used?
    fn stop_listener(&mut self, socket_adder: &SocketAddr) {
        if let Some(tcp_connection_handler_holder) = self.tcp_listeners.remove(socket_adder) {
            tcp_connection_handler_holder.stop();
        }
    }

    fn listener_on_connection(&mut self, socket_adder: SocketAddr, writer: ChannelTcpWriter<Factory>, reader: ChannelTcpReader<Factory>) {
        if let Some(mut tcp_connection_handler_holder) = self.tcp_listeners.remove(&socket_adder) {
            if let Some(tcp_connection_handler_holder) = tcp_connection_handler_holder.on_connection(writer, reader) {
                self.start_listener(socket_adder, tcp_connection_handler_holder);
            }
        }
    }

    fn on_send_to_tcp_listener(&mut self, socket_adder: SocketAddr) {
        if let Some(mut tcp_connection_handler_holder) = self.tcp_listeners.remove(&socket_adder) {
            if let Some(tcp_connection_handler_holder) = tcp_connection_handler_holder.on_send() {
                self.start_listener(socket_adder, tcp_connection_handler_holder);
            }
        }
    }

    fn connect_tcp(&self, factory: &Factory, client_socket_addr: SocketAddr, server_socket_addr: SocketAddr) -> Result<(ChannelTcpWriter<Factory>, ChannelTcpReader<Factory>), Error> {

        if let Some(tcp_listener)  = self.tcp_listeners.get(&server_socket_addr) {

            let (write_server_to_client, read_server_to_client) = Self::new_tcp_channel(factory, client_socket_addr);
            let (write_client_to_server, read_client_to_server) = Self::new_tcp_channel(factory, server_socket_addr);


            /*
            let tcp_listener = tcp_listener.lock().unwrap();

            if let Some(tcp_listener) = tcp_listener.take() {

            }

            info!("{:?} connected (TCP) to {:?}", client_socket_addr, server_socket_addr);

            let (write_server_to_client, read_server_to_client) = Self::new_tcp_channel(factory, client_socket_addr);
            let (write_client_to_server, read_client_to_server) = Self::new_tcp_channel(factory, server_socket_addr);

            let mut x = tcp_listener.lock().unwrap();
            x.*/
        }

        info!("{:?} tried to connect (TCP) to {:?} but there is no listener at that SocketAddr.", client_socket_addr, server_socket_addr);
        return Err(Error::from(ErrorKind::ConnectionRefused));
        todo!()
    }

    fn new_tcp_channel(factory: &Factory, peer_socket_addr: SocketAddr) -> (ChannelTcpWriter<Factory>, ChannelTcpReader<Factory>) {

        let (sender, receiver) = factory.new_channel::<Vec<u8>>().take();
        let reader = ChannelTcpReader::new(peer_socket_addr, receiver);
        let writer = ChannelTcpWriter::new(peer_socket_addr, sender);
        return (writer, reader);
    }
}

