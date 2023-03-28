use std::io::Error;
use std::net::{TcpListener, ToSocketAddrs};
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::ip::realtcpstream::RealTcpStream;
use crate::ip::TcpListenerTrait;

pub struct RealTcpListener {
    tcp_listener: TcpListener
}

impl RealTcpListener {
    pub fn bind(socket_addr: impl ToSocketAddrs) -> Result<Self, Error>{
        return Ok(Self {
            tcp_listener: TcpListener::bind(socket_addr)?
        });
    }
}

impl TcpListenerTrait for RealTcpListener {
    type TcpStream<ReadType: Serialize + DeserializeOwned + Send, WriteType: Serialize + DeserializeOwned + Send> = RealTcpStream<ReadType, WriteType>;

    fn accept<ReadType: Serialize + DeserializeOwned + Send, WriteType: Serialize + DeserializeOwned + Send>(&self) -> Result<Self::TcpStream<ReadType, WriteType>, Error> {
        let (tcp_stream, remote_peer_socket_addr) = self.tcp_listener.accept()?;

        return Ok(RealTcpStream::new(tcp_stream, remote_peer_socket_addr));
    }
}