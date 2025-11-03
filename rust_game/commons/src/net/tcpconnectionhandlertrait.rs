use crate::{
    factory::FactoryTrait,
    net::TcpStream,
};
use std::{
    net::SocketAddr,
    ops::ControlFlow,
};

//TODO: get rid of this trait
pub trait TcpConnectionHandlerTrait<Factory: FactoryTrait>: Send + 'static {
    fn on_bind(&mut self, _socket_addr: SocketAddr) {}

    fn on_connection(
        &mut self,
        tcp_stream: TcpStream,
        tcp_receiver: Factory::TcpReader,
    ) -> ControlFlow<()>;
}

pub struct TcpConnectionHandler<Factory: FactoryTrait> {
    on_bind: Box<dyn FnMut(SocketAddr) + Send + 'static>,
    on_connection:
        Box<dyn FnMut(TcpStream, Factory::TcpReader) -> ControlFlow<()> + Send + 'static>,
}

impl<Factory: FactoryTrait> TcpConnectionHandler<Factory> {
    pub fn new() -> Self {
        return Self {
            on_bind: Box::new(|_| {}),
            on_connection: Box::new(|_, _| ControlFlow::Continue(())),
        };
    }

    pub fn set_on_bind(&mut self, on_bind: impl FnMut(SocketAddr) + Send + 'static) {
        self.on_bind = Box::new(on_bind);
    }

    pub fn set_on_connection(
        &mut self,
        on_connection: impl FnMut(TcpStream, Factory::TcpReader) -> ControlFlow<()> + Send + 'static,
    ) {
        self.on_connection = Box::new(on_connection);
    }
}

impl<Factory: FactoryTrait> TcpConnectionHandlerTrait<Factory> for TcpConnectionHandler<Factory> {
    fn on_bind(&mut self, socket_addr: SocketAddr) {
        return (self.on_bind)(socket_addr);
    }

    fn on_connection(
        &mut self,
        tcp_stream: TcpStream,
        tcp_receiver: <Factory as FactoryTrait>::TcpReader,
    ) -> ControlFlow<()> {
        return (self.on_connection)(tcp_stream, tcp_receiver);
    }
}
