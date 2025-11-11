use crate::net::{
    TcpReadHandlerTrait,
    NET_POLLING_PERIOD,
};
use crate::threading::channel::Receiver;
use crate::threading::eventhandling::EventOrStopThread;
use crate::threading::AsyncJoinCallBackTrait;
use rmp_serde::encode::Error as EncodeError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::io::{
    Error,
    Write,
};
use std::net::SocketAddr;

#[derive(Debug)]
pub struct RealTcpStream {
    tcp_stream: std::net::TcpStream,
    remote_peer_socket_addr: SocketAddr,
}

impl RealTcpStream {
    pub fn new(tcp_stream: std::net::TcpStream, remote_peer_socket_addr: SocketAddr) -> Self {
        tcp_stream
            .set_read_timeout(Some(NET_POLLING_PERIOD.to_duration().unwrap()))
            .unwrap();

        return Self {
            tcp_stream,
            remote_peer_socket_addr,
        };
    }

    pub fn get_peer_addr(&self) -> &SocketAddr {
        return &self.remote_peer_socket_addr;
    }

    pub fn get_local_addr(&self) -> Result<SocketAddr, Error> {
        return self.tcp_stream.local_addr();
    }

    pub fn try_clone(&self) -> Result<Self, Error> {
        return Ok(Self {
            tcp_stream: self.tcp_stream.try_clone()?,
            remote_peer_socket_addr: self.remote_peer_socket_addr.clone(),
        });
    }

    pub fn take_std_net_tcp_reader(self) -> std::net::TcpStream {
        return self.tcp_stream;
    }

    pub fn write<T: Serialize + DeserializeOwned>(&mut self, write: &T) -> Result<(), EncodeError> {
        return rmp_serde::encode::write(&mut self.tcp_stream, &write);
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        return self.tcp_stream.flush();
    }

    pub fn spawn_real_tcp_reader<T: TcpReadHandlerTrait>(
        self,
        thread_name: String,
        receiver: Receiver<EventOrStopThread<()>>,
        tcp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T>,
    ) -> Result<(), Error> {
        return receiver.spawn_real_tcp_reader(thread_name, self, tcp_read_handler, join_call_back);
    }
}
