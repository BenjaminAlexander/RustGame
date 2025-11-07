use crate::factory::FactoryTrait;
use crate::net::{
    RealTcpStream,
    RealUdpSocket,
    TcpReader,
    TcpStream,
    UdpSocket,
};
use crate::threading::channel::{
    Channel,
    RealReceiver,
    RealSender,
    SendMetaData,
};
use crate::time::TimeSource;
use std::io::Error;
use std::net::SocketAddr;
use std::sync::mpsc;

#[derive(Clone)]
pub struct RealFactory {
    time_source: TimeSource,
}

impl RealFactory {
    pub fn new() -> Self {
        return Self {
            time_source: TimeSource::new(),
        };
    }
}

impl FactoryTrait for RealFactory {
    fn get_time_source(&self) -> &TimeSource {
        return &self.time_source;
    }

    fn new_channel<T: Send>(&self) -> Channel<T> {
        let (sender, receiver) = mpsc::channel::<(SendMetaData, T)>();
        let sender = RealSender::new(self.time_source.clone(), sender);
        let receiver = RealReceiver::new(self.time_source.clone(), receiver);
        return Channel::new(sender, receiver);
    }

    fn connect_tcp(&self, socket_addr: SocketAddr) -> Result<(TcpStream, TcpReader), Error> {
        let net_tcp_stream = std::net::TcpStream::connect(socket_addr.clone())?;
        let real_tcp_stream = RealTcpStream::new(net_tcp_stream, socket_addr.clone());
        let tcp_stream = TcpStream::new(real_tcp_stream.try_clone()?);
        let tcp_reader = TcpReader::new(real_tcp_stream);
        return Ok((tcp_stream, tcp_reader));
    }

    fn bind_udp_socket(&self, socket_addr: SocketAddr) -> Result<UdpSocket, Error> {
        return Ok(UdpSocket::new(RealUdpSocket::bind(socket_addr)?));
    }
}
