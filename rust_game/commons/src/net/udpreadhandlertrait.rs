use std::net::SocketAddr;
use std::ops::ControlFlow;

use crate::threading::channel::ReceiveMetaData;

pub trait UdpReadHandlerTrait: Send + Sized + 'static {
    fn on_read(self, peer_addr: SocketAddr, buf: &[u8]) -> ControlFlow<(), Self>;

    //TODO: this needs some documentation
    fn on_channel_disconnected(self) {
        //no-op default implementation
    }

    //TODO: this needs some documentation
    fn on_read_error(self) {
        //no-op default implementation
    }

    //TODO: this needs some documentation
    fn on_stop(self, _receive_meta_data: ReceiveMetaData) {
        //no-op default implementation
    }
}

pub struct UdpReadHandler {
    on_read: Box<dyn FnMut(SocketAddr, &[u8]) -> ControlFlow<()> + Send + 'static>,
    on_channel_disconnected: Box<dyn FnOnce() + Send + 'static>,
    on_read_error: Box<dyn FnOnce() + Send + 'static>,
    on_stop: Box<dyn FnOnce() + Send + 'static>,
}

impl UdpReadHandler {
    pub fn new(on_read: impl FnMut(SocketAddr, &[u8]) -> ControlFlow<()> + Send + 'static) -> Self {
        return UdpReadHandler {
            on_read: Box::new(on_read),
            on_channel_disconnected: Box::new(||{}),
            on_read_error: Box::new(||{}),
            on_stop: Box::new(||{}),
        };
    }

    pub fn set_on_channel_disconnected(&mut self, func: impl FnOnce() + Send + 'static) {
        self.on_channel_disconnected = Box::new(func);
    }

    pub fn set_on_read_error(&mut self, func: impl FnOnce() + Send + 'static) {
        self.on_read_error = Box::new(func);
    }

    pub fn set_on_stop(&mut self, func: impl FnOnce() + Send + 'static) {
        self.on_stop = Box::new(func);
    }
}

impl UdpReadHandlerTrait for UdpReadHandler {

    fn on_read(mut self, peer_addr: SocketAddr, buf: &[u8]) -> ControlFlow<(), Self> {
        match (self.on_read)(peer_addr, buf) {
            ControlFlow::Continue(()) => ControlFlow::Continue(self),
            ControlFlow::Break(()) => ControlFlow::Break(()),
        }
    }

    fn on_channel_disconnected(self) {
        (self.on_channel_disconnected)();
    }

    fn on_read_error(self) {
        (self.on_read_error)();
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) {
        (self.on_stop)();
    }
}
