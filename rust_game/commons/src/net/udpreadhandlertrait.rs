use std::net::SocketAddr;
use std::ops::ControlFlow;

pub trait UdpReadHandlerTrait: Send + 'static {

    fn on_read(&mut self, peer_addr: SocketAddr, buf: &[u8]) -> ControlFlow<()>;

}