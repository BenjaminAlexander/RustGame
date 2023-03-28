use std::fmt::Debug;
use std::io::{Error, Write};
use std::marker::PhantomData;
use std::net::{SocketAddr, TcpStream};
use serde::de::DeserializeOwned;
use serde::Serialize;
use rmp_serde::decode::Error as DecodeError;
use rmp_serde::encode::Error as EncodeError;
use crate::ip::tcpstreamtrait::TcpStreamTrait;

#[derive(Debug)]
pub struct RealTcpStream<ReadType: Serialize + DeserializeOwned, WriteType: Serialize + DeserializeOwned> {
    tcp_stream: TcpStream,
    remote_peer_socket_addr: SocketAddr,
    phantom: PhantomData<(ReadType, WriteType)>
}

impl<ReadType: Serialize + DeserializeOwned, WriteType: Serialize + DeserializeOwned> RealTcpStream<ReadType, WriteType> {

    pub fn new(tcp_stream: TcpStream, remote_peer_socket_addr: SocketAddr) -> Self {
        return Self {
            tcp_stream,
            remote_peer_socket_addr,
            phantom: PhantomData::default()
        };
    }
}

impl<ReadType: Serialize + DeserializeOwned + Send, WriteType: Serialize + DeserializeOwned + Send> TcpStreamTrait<ReadType, WriteType> for RealTcpStream<ReadType, WriteType> {

    fn read(&self) -> Result<ReadType, DecodeError> {
        return rmp_serde::from_read(&self.tcp_stream);
    }

    fn write(&mut self, write: &WriteType) -> Result<(), EncodeError> {
        return rmp_serde::encode::write(&mut self.tcp_stream, &write);
    }

    fn flush(&mut self) -> Result<(), Error> {
        return self.tcp_stream.flush();
    }

    fn get_peer_addr(&self) -> &SocketAddr {
        return &self.remote_peer_socket_addr;
    }

    fn try_clone(&self) -> Result<Self, Error> {
        return Ok(Self {
            tcp_stream: self.tcp_stream.try_clone()?,
            remote_peer_socket_addr: self.remote_peer_socket_addr.clone(),
            phantom: PhantomData::default()
        });
    }
}