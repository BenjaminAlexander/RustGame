use crate::factory::FactoryTrait;
use crate::net::{
    RealTcpStream,
    RealUdpSocket,
    TcpConnectionHandlerTrait,
    TcpListenerEventHandler,
    TcpReadHandlerTrait,
    TcpReaderEventHandler,
    TcpStream,
    UdpReadHandlerTrait,
    UdpReaderEventHandler,
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
    EventHandlerThread,
    EventHandlerTrait,
    EventOrStopThread,
};
use crate::threading::{
    channel,
    AsyncJoinCallBackTrait,
};
use crate::time::TimeSource;
use std::io::Error;
use std::net::{
    SocketAddr,
    TcpListener,
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
    type Sender<T: Send> = RealSender<Self, T>;
    type Receiver<T: Send> = RealReceiver<Self, T>;

    type TcpReader = RealTcpStream;

    type UdpSocket = RealUdpSocket;

    fn get_time_source(&self) -> &TimeSource {
        return &self.time_source;
    }

    fn new_channel<T: Send>(&self) -> Channel<Self, T> {
        let (sender, receiver) = mpsc::channel::<(SendMetaData, T)>();
        let sender = RealSender::new(self.clone(), sender);
        let receiver = RealReceiver::new(self.clone(), receiver);
        return Channel::new(sender, receiver);
    }

    fn spawn_event_handler<U: EventHandlerTrait>(
        &self,
        thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<U::Event>>,
        event_handler: U,
        join_call_back: impl AsyncJoinCallBackTrait<Self, U::ThreadReturn>,
    ) -> std::io::Result<EventHandlerSender<Self, U::Event>> {
        let (thread_builder, channel) = thread_builder.take();
        let (sender, receiver) = channel.take();

        let thread = EventHandlerThread::new(receiver, event_handler);

        thread_builder.spawn_thread(thread, join_call_back)?;

        return Ok(sender);
    }

    fn spawn_tcp_listener<T: TcpConnectionHandlerTrait<Self>>(
        &self,
        thread_builder: channel::ChannelThreadBuilder<Self, EventOrStopThread<()>>,
        socket_addr: SocketAddr,
        mut tcp_connection_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Self, T>,
    ) -> Result<EventHandlerSender<Self, ()>, Error> {
        let tcp_listener = TcpListener::bind(socket_addr)?;

        tcp_connection_handler.on_bind(tcp_listener.local_addr()?);

        let event_handler = TcpListenerEventHandler::new(tcp_listener, tcp_connection_handler)?;
        return thread_builder.spawn_event_handler(event_handler, join_call_back);
    }

    fn connect_tcp(&self, socket_addr: SocketAddr) -> Result<(TcpStream, Self::TcpReader), Error> {
        let net_tcp_stream = std::net::TcpStream::connect(socket_addr.clone())?;
        let real_tcp_stream = RealTcpStream::new(net_tcp_stream, socket_addr.clone());
        return Ok((
            TcpStream::new(real_tcp_stream.try_clone()?),
            real_tcp_stream,
        ));
    }

    fn spawn_tcp_reader<T: TcpReadHandlerTrait>(
        &self,
        thread_builder: channel::ChannelThreadBuilder<Self, EventOrStopThread<()>>,
        tcp_reader: Self::TcpReader,
        tcp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Self, T>,
    ) -> Result<EventHandlerSender<Self, ()>, Error> {
        let event_handler = TcpReaderEventHandler::new(tcp_reader, tcp_read_handler);
        return thread_builder.spawn_event_handler(event_handler, join_call_back);
    }

    fn bind_udp_socket(&self, socket_addr: SocketAddr) -> Result<Self::UdpSocket, Error> {
        return RealUdpSocket::bind(socket_addr);
    }

    fn spawn_udp_reader<T: UdpReadHandlerTrait>(
        &self,
        thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<()>>,
        udp_socket: Self::UdpSocket,
        udp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Self, T>,
    ) -> Result<EventHandlerSender<Self, ()>, Error> {
        let event_handler = UdpReaderEventHandler::new(udp_socket, udp_read_handler);
        return thread_builder.spawn_event_handler(event_handler, join_call_back);
    }
}
