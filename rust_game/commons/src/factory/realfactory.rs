use crate::factory::FactoryTrait;
use crate::net::{
    RealTcpStream, RealUdpSocket, TcpReceiver, TcpStream, UdpReadHandlerTrait, UdpReaderEventHandler
};
use crate::threading::channel::{
    Channel,
    ChannelThreadBuilder,
    RealReceiver,
    RealSender,
    SendMetaData,
};
use crate::threading::eventhandling::{
    EventHandlerSender,
    EventOrStopThread,
};
use crate::threading::AsyncJoinCallBackTrait;
use crate::time::TimeSource;
use std::io::Error;
use std::net::{
    SocketAddr,
};
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
    
    type UdpSocket = RealUdpSocket;

    fn get_time_source(&self) -> &TimeSource {
        return &self.time_source;
    }

    fn new_channel<T: Send>(&self) -> Channel<T> {
        let (sender, receiver) = mpsc::channel::<(SendMetaData, T)>();
        let sender = RealSender::new(self.time_source.clone(), sender);
        let receiver = RealReceiver::new(self.time_source.clone(), receiver);
        return Channel::new(sender, receiver);
    }

    fn connect_tcp(&self, socket_addr: SocketAddr) -> Result<(TcpStream, TcpReceiver), Error> {
        let net_tcp_stream = std::net::TcpStream::connect(socket_addr.clone())?;
        let real_tcp_stream = RealTcpStream::new(net_tcp_stream, socket_addr.clone());
        let tcp_stream = TcpStream::new(real_tcp_stream.try_clone()?);
        let tcp_reader = TcpReceiver::new(real_tcp_stream);
        return Ok((
            tcp_stream,
            tcp_reader,
        ));
    }

    fn bind_udp_socket(&self, socket_addr: SocketAddr) -> Result<Self::UdpSocket, Error> {
        return RealUdpSocket::bind(socket_addr);
    }

    fn spawn_udp_reader<T: UdpReadHandlerTrait>(
        &self,
        thread_builder: ChannelThreadBuilder<EventOrStopThread<()>>,
        udp_socket: Self::UdpSocket,
        udp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T>,
    ) -> Result<EventHandlerSender<()>, Error> {
        let event_handler = UdpReaderEventHandler::new(udp_socket, udp_read_handler);
        return self.spawn_event_handler(thread_builder, event_handler, join_call_back);
    }
}
