use std::io::Error;
use crate::ip::tcpstreamtrait::TcpStreamTrait;

pub trait TcpListenerTrait {
    type TcpStream<T>: TcpStreamTrait<T=T>;

    fn accept<T>(&self) -> Result<Self::TcpStream<T>, Error>;
}