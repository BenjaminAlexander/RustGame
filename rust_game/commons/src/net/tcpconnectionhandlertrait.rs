use crate::{
    net::{
        TcpReader,
        TcpStream,
    },
    threading::channel::ReceiveMetaData,
};
use std::{
    net::SocketAddr,
    ops::ControlFlow,
};

//TODO: get rid of this trait
pub trait TcpConnectionHandlerTrait: Send + Sized + 'static {
    fn on_bind(&mut self, _socket_addr: SocketAddr) {
        //no-op default
    }

    fn on_connection(self, tcp_stream: TcpStream, tcp_receiver: TcpReader)
        -> ControlFlow<(), Self>;

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) {
        //no-op default implementation
    }

    //TODO: this needs some documentation
    fn on_channel_disconnected(self) {
        //no-op default implementation
    }
}

pub struct TcpConnectionHandler {
    on_bind: Box<dyn FnMut(SocketAddr) + Send + 'static>,
    on_connection: Box<dyn FnMut(TcpStream, TcpReader) -> ControlFlow<()> + Send + 'static>,
    on_channel_disconnected: Box<dyn FnOnce() + Send + 'static>,
    on_stop: Box<dyn FnOnce() + Send + 'static>,
}

impl TcpConnectionHandler {
    pub fn new() -> Self {
        return Self {
            on_bind: Box::new(|_| {}),
            on_connection: Box::new(|_, _| ControlFlow::Continue(())),
            on_channel_disconnected: Box::new(||{}),
            on_stop: Box::new(||{}),
        };
    }

    pub fn set_on_bind(&mut self, on_bind: impl FnMut(SocketAddr) + Send + 'static) {
        self.on_bind = Box::new(on_bind);
    }

    pub fn set_on_connection(
        &mut self,
        on_connection: impl FnMut(TcpStream, TcpReader) -> ControlFlow<()> + Send + 'static,
    ) {
        self.on_connection = Box::new(on_connection);
    }

    pub fn set_on_channel_disconnected(&mut self, func: impl FnOnce() + Send + 'static) {
        self.on_channel_disconnected = Box::new(func);
    }

    pub fn set_on_stop(&mut self, func: impl FnOnce() + Send + 'static) {
        self.on_stop = Box::new(func);
    }
}

impl TcpConnectionHandlerTrait for TcpConnectionHandler {
    fn on_bind(&mut self, socket_addr: SocketAddr) {
        return (self.on_bind)(socket_addr);
    }

    fn on_connection(
        mut self,
        tcp_stream: TcpStream,
        tcp_receiver: TcpReader,
    ) -> ControlFlow<(), Self> {
        match (self.on_connection)(tcp_stream, tcp_receiver) {
            ControlFlow::Continue(()) => ControlFlow::Continue(self),
            ControlFlow::Break(()) => ControlFlow::Break(()),
        }
    }

    fn on_channel_disconnected(self) {
        (self.on_channel_disconnected)();
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) {
        (self.on_stop)();
    }
}
