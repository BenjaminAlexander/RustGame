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

impl<T: FnMut(SocketAddr, &[u8]) -> ControlFlow<()> + Send + 'static> UdpReadHandlerTrait for T {
    fn on_read(mut self, peer_addr: SocketAddr, buf: &[u8]) -> ControlFlow<(), Self> {
        match (self)(peer_addr, buf) {
            ControlFlow::Continue(()) => ControlFlow::Continue(self),
            ControlFlow::Break(()) => ControlFlow::Break(()),
        }
    }
}
