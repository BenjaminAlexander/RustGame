use crate::real_time::net::tcp::{
    TcpReader,
    TcpStream,
};
use crate::real_time::net::udp::UdpSocket;
use crate::real_time::real::net::tcp::RealTcpStream;
use crate::real_time::real::net::udp::RealUdpSocket;
use crate::real_time::real::{
    RealReceiver,
    RealSender,
};
use crate::real_time::receiver::ReceiverImplementation;
use crate::real_time::sender::SenderImplementation;
use crate::real_time::{
    FactoryTrait,
    Receiver,
    SendMetaData,
    Sender,
    TimeSource,
};
use std::io::Error;
use std::net::SocketAddr;
use std::sync::mpsc;

//TODO: rename trait and file
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

    fn new_channel<T: Send>(&self) -> (Sender<T>, Receiver<T>) {
        let (sender, receiver) = mpsc::channel::<(SendMetaData, T)>();
        let real_sender = RealSender::new(self.time_source.clone(), sender);
        let real_receiver = RealReceiver::new(self.time_source.clone(), receiver);
        let sender = Sender::new(SenderImplementation::Real(real_sender));
        let receiver = Receiver::new(ReceiverImplementation::Real(real_receiver));
        return (sender, receiver);
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
