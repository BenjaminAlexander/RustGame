use rmp_serde::encode::Error as EncodeError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::Error;
use std::net::SocketAddr;

use crate::net::RealTcpStream;
use crate::single_threaded_simulator::net::ChannelTcpWriter;

enum Implementation {
    Real(RealTcpStream),

    //TODO: conditionally compile
    Simulated(ChannelTcpWriter),
}

pub struct TcpStream {
    implementation: Implementation,
}

impl TcpStream {
    pub fn new(real_tcp_stream: RealTcpStream) -> Self {
        return Self {
            implementation: Implementation::Real(real_tcp_stream),
        };
    }

    pub fn new_simulated(channel_tcp_writer: ChannelTcpWriter) -> Self {
        return Self {
            implementation: Implementation::Simulated(channel_tcp_writer),
        };
    }

    pub fn write<T: Serialize + DeserializeOwned>(&mut self, write: &T) -> Result<(), EncodeError> {
        return match &mut self.implementation {
            Implementation::Real(real_tcp_stream) => real_tcp_stream.write(write),
            Implementation::Simulated(channel_tcp_writer) => channel_tcp_writer.write(write),
        };
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        return match &mut self.implementation {
            Implementation::Real(real_tcp_stream) => real_tcp_stream.flush(),
            Implementation::Simulated(channel_tcp_writer) => channel_tcp_writer.flush(),
        };
    }

    pub fn get_peer_addr(&self) -> &SocketAddr {
        return match &self.implementation {
            Implementation::Real(real_tcp_stream) => real_tcp_stream.get_peer_addr(),
            Implementation::Simulated(channel_tcp_writer) => channel_tcp_writer.get_peer_addr(),
        };
    }

    pub fn try_clone(&self) -> Result<Self, Error> {
        return match &self.implementation {
            Implementation::Real(real_tcp_stream) => Ok(Self {
                implementation: Implementation::Real(real_tcp_stream.try_clone()?),
            }),
            Implementation::Simulated(channel_tcp_writer) => Ok(Self {
                implementation: Implementation::Simulated(channel_tcp_writer.clone()),
            }),
        };
    }
}
