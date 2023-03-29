use std::ops::ControlFlow;
use crate::net::TcpStreamTrait;

pub trait TcpConnectionHandler: Send + 'static {
    type TcpStream: TcpStreamTrait;

    fn on_connection(&mut self, tcp_stream: Self::TcpStream) -> ControlFlow<()>;
}