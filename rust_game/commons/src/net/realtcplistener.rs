use std::io::Error;
use std::marker::PhantomData;
use std::net::{TcpListener, ToSocketAddrs};
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::net::realtcpstream::RealTcpStream;
use crate::net::TcpListenerTrait;

pub struct RealTcpListener<ReadType: Serialize + DeserializeOwned + Send, WriteType: Serialize + DeserializeOwned + Send> {
    tcp_listener: TcpListener,
    phantom: PhantomData<(ReadType, WriteType)>
}

impl<ReadType: Serialize + DeserializeOwned + Send, WriteType: Serialize + DeserializeOwned + Send> RealTcpListener<ReadType, WriteType> {
    pub fn bind(socket_addr: impl ToSocketAddrs) -> Result<Self, Error>{
        return Ok(Self {
            tcp_listener: TcpListener::bind(socket_addr)?,
            phantom: PhantomData::default()
        });
    }
}

impl<ReadType: Serialize + DeserializeOwned + Send, WriteType: Serialize + DeserializeOwned + Send> TcpListenerTrait for RealTcpListener<ReadType, WriteType> {
    type ReadType = ReadType;
    type WriteType = WriteType;
    type TcpStream = RealTcpStream<ReadType, WriteType>;

    fn accept(&self) -> Result<Self::TcpStream, Error> {
        let (tcp_stream, remote_peer_socket_addr) = self.tcp_listener.accept()?;

        return Ok(RealTcpStream::new(tcp_stream, remote_peer_socket_addr));
    }
}