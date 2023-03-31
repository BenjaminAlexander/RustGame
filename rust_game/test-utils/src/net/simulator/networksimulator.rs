use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use commons::factory::FactoryTrait;
use commons::net::TcpConnectionHandlerTrait;
use commons::threading::{AsyncJoinCallBackTrait, ThreadBuilder};
use crate::net::simulator::hostsimulator::HostSimulator;
use crate::net::simulator::tcpconnectionhandlerholdertrait;
use crate::net::simulator::tcpconnectionhandlerholdertrait::TcpConnectionHandlerHolderTrait;

#[derive(Clone)]
pub struct NetworkSimulator {
    internal: Arc<Mutex<Internal>>
}

impl NetworkSimulator {
    pub fn new() -> Self {

        let internal = Internal {
            tcp_listeners: HashMap::new()
        };

        return Self {
            internal: Arc::new(Mutex::new(internal))
        }
    }

    pub fn new_host(&self, ip_addr: IpAddr) -> HostSimulator {
        return HostSimulator::new(self.clone(), ip_addr);
    }

    pub fn start_listener<
        Factory: FactoryTrait,
        TcpConnectionHandler: TcpConnectionHandlerTrait<TcpSender=Factory::TcpSender, TcpReceiver=Factory::TcpReceiver>
    >(
        &self,
        socket_adder: SocketAddr,
        thread_builder: ThreadBuilder<Factory>,
        connection_handler: TcpConnectionHandler,
        join_call_back: impl AsyncJoinCallBackTrait<Factory, TcpConnectionHandler>
    ) {

        let tcp_connection_handler_holder = tcpconnectionhandlerholdertrait::new(
            thread_builder,
            connection_handler,
            join_call_back
        );

        self.internal.lock().unwrap().start_listener(socket_adder, tcp_connection_handler_holder);
    }

    pub fn stop_listener(&self, socket_adder: &SocketAddr) {
        self.internal.lock().unwrap().stop_listener(socket_adder);
    }
}

struct Internal {
    tcp_listeners: HashMap<SocketAddr, Box<dyn TcpConnectionHandlerHolderTrait + Send>>
}

impl Internal {

    fn start_listener(&mut self, socket_adder: SocketAddr, tcp_connection_handler_holder: Box<dyn TcpConnectionHandlerHolderTrait + Send>) {
        self.tcp_listeners.insert(socket_adder, tcp_connection_handler_holder);
    }

    fn stop_listener(&mut self, socket_adder: &SocketAddr) {
        self.tcp_listeners.remove(socket_adder);
    }
}

