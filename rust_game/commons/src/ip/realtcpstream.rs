use std::marker::PhantomData;
use std::net::{SocketAddr, TcpStream};
use serde::de::DeserializeOwned;
use serde::Serialize;
use rmp_serde::decode::Error as DecodeError;
use rmp_serde::encode::Error as EncodeError;
use crate::ip::tcplistenertrait::TcpListenerTrait;
use crate::ip::tcpstreamtrait::TcpStreamTrait;

pub struct RealTcpStream<T: Serialize + DeserializeOwned> {
    tcp_stream: TcpStream,
    remote_peer_socket_addr: SocketAddr,
    phantom: PhantomData<T>
}

impl<T: Serialize + DeserializeOwned> RealTcpStream<T> {

    pub fn new(tcp_stream: TcpStream, remote_peer_socket_addr: SocketAddr) -> Self {
        return Self {
            tcp_stream,
            remote_peer_socket_addr,
            phantom: PhantomData::default()
        };
    }
}

impl<T: Serialize + DeserializeOwned> TcpStreamTrait for RealTcpStream<T> {
    type T = T;

    fn read(&self) -> Result<T, DecodeError> {
        return rmp_serde::from_read(&self.tcp_stream);
    }

    fn write(&mut self, t: &T) -> Result<(), EncodeError> {
        return rmp_serde::encode::write(&mut self.tcp_stream, &t);
    }
}