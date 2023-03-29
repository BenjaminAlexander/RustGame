use std::io::Error;
use crate::net::tcpstreamtrait::TcpStreamTrait;

pub trait TcpListenerTrait: Send {
    type TcpStream: TcpStreamTrait;

    fn accept(&self) -> Result<Self::TcpStream, Error>;
}