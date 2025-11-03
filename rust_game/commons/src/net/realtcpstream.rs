use crate::net::TcpWriterTrait;
use log::warn;
use rmp_serde::decode::Error as DecodeError;
use rmp_serde::encode::Error as EncodeError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::io::{
    Error,
    ErrorKind,
    Write,
};
use std::net::{
    SocketAddr,
    TcpStream,
};

use super::resetablereader::ResetableReader;
use super::TCP_POLLING_PERIOD;

#[derive(Debug)]
pub struct RealTcpStream {
    tcp_stream: TcpStream,
    remote_peer_socket_addr: SocketAddr,
}

impl RealTcpStream {
    pub fn new(tcp_stream: TcpStream, remote_peer_socket_addr: SocketAddr) -> Self {
        tcp_stream
            .set_read_timeout(Some(TCP_POLLING_PERIOD.to_duration().unwrap()))
            .unwrap();

        return Self {
            tcp_stream,
            remote_peer_socket_addr,
        };
    }

    fn get_peer_addr(&self) -> &SocketAddr {
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

    pub fn to_deserializer(self) -> TcpDeserializer {
        return TcpDeserializer {
            resetable_reader: ResetableReader::new(self.tcp_stream),
        };
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

pub struct TcpDeserializer {
    resetable_reader: ResetableReader<TcpStream>,
}

pub enum TcpReadResult<T> {
    Ok(T),
    TimedOut,
    Err,
}

impl TcpDeserializer {
    pub fn read<T: Serialize + DeserializeOwned>(&mut self) -> TcpReadResult<T> {
        let result = rmp_serde::decode::from_read(&mut self.resetable_reader);

        return match result {
            Ok(value) => {
                self.resetable_reader.drop_read_bytes();
                TcpReadResult::Ok(value)
            }
            Err(DecodeError::InvalidMarkerRead(ref error))
                if error.kind() == ErrorKind::TimedOut || error.kind() == ErrorKind::WouldBlock =>
            {
                self.resetable_reader.reset_cursor();
                TcpReadResult::TimedOut
            }
            Err(error) => {
                self.resetable_reader.drop_read_bytes();
                warn!("Error on TCP read: {:?}", error);
                TcpReadResult::Err
            }
        };
    }
}
