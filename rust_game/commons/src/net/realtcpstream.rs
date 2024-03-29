use std::fmt::Debug;
use std::io::{Error, Write};
use std::net::{SocketAddr, TcpStream};
use serde::de::DeserializeOwned;
use serde::Serialize;
use rmp_serde::decode::Error as DecodeError;
use rmp_serde::encode::Error as EncodeError;
use crate::net::TcpWriterTrait;

#[derive(Debug)]
pub struct RealTcpStream {
    tcp_stream: TcpStream,
    remote_peer_socket_addr: SocketAddr,
}

impl RealTcpStream {

    pub fn new(tcp_stream: TcpStream, remote_peer_socket_addr: SocketAddr) -> Self {
        return Self {
            tcp_stream,
            remote_peer_socket_addr
        };
    }

    fn get_peer_addr(&self) -> &SocketAddr {
        return &self.remote_peer_socket_addr;
    }

    pub fn try_clone(&self) -> Result<Self, Error> {
        return Ok(Self {
            tcp_stream: self.tcp_stream.try_clone()?,
            remote_peer_socket_addr: self.remote_peer_socket_addr.clone()
        });
    }

    pub fn read<T: Serialize + DeserializeOwned>(&mut self) -> Result<T, DecodeError> {
        return rmp_serde::from_read(&self.tcp_stream);
    }
}

impl TcpWriterTrait for RealTcpStream {

    fn write<T: Serialize + DeserializeOwned>(&mut self, write: &T) -> Result<(), EncodeError> {
        return rmp_serde::encode::write(&mut self.tcp_stream, &write);
    }

    fn flush(&mut self) -> Result<(), Error> {
        return self.tcp_stream.flush();
    }

    fn get_peer_addr(&self) -> &SocketAddr {
        return RealTcpStream::get_peer_addr(self);
    }
}