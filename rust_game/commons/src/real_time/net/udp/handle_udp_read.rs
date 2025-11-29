use std::net::SocketAddr;
use std::ops::ControlFlow;

pub trait HandleUdpRead: Send + 'static {
    fn on_read(&mut self, peer_addr: SocketAddr, buf: &[u8]) -> ControlFlow<()>;
}

impl<T: FnMut(SocketAddr, &[u8]) -> ControlFlow<()> + Send + 'static> HandleUdpRead for T {
    fn on_read(&mut self, peer_addr: SocketAddr, buf: &[u8]) -> ControlFlow<()> {
        return (self)(peer_addr, buf);
    }
}
