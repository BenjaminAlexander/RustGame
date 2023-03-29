use std::ops::ControlFlow;
use crate::net::TcpStreamTrait;

pub trait TcpConnectionHandlerTrait: Send + 'static {
    type TcpStream: TcpStreamTrait;

    fn on_connection(&mut self, tcp_stream: Self::TcpStream) -> ControlFlow<()>;
}