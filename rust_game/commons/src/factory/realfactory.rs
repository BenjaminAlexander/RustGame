use std::io::Error;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc;
use std::time::SystemTime;
use crate::factory::FactoryTrait;
use crate::net::{RealTcpStream, RealUdpSocket, TcpConnectionHandlerTrait, TcpListenerEventHandler, TcpReaderEventHandler, TcpReadHandlerTrait, UdpReaderEventHandler, UdpReadHandlerTrait};
use crate::threading::channel::{Channel, ChannelThreadBuilder, RealSender, RealReceiver, SendMetaData};
use crate::threading::eventhandling::{EventHandlerThread, EventHandlerTrait, EventOrStopThread, EventHandlerSender};
use crate::threading::{AsyncJoinCallBackTrait, channel};
use crate::time::TimeValue;

#[derive(Clone, Copy)]
pub struct RealFactory {

}

impl RealFactory {
    pub fn new() -> Self {
        return Self {};
    }
}

impl FactoryTrait for RealFactory {
    type Sender<T: Send> = RealSender<Self, T>;
    type Receiver<T: Send> = RealReceiver<Self, T>;

    type TcpWriter = RealTcpStream;
    type TcpReader = RealTcpStream;

    type UdpSocket = RealUdpSocket;
    
    fn now(&self) -> TimeValue {
        return TimeValue::from_system_time(&SystemTime::now()).unwrap();
    }

    fn new_channel<T: Send>(&self) -> Channel<Self, T> {
        let (sender, receiver) = mpsc::channel::<(SendMetaData, T)>();
        let sender = RealSender::new(self.clone(), sender);
        let receiver = RealReceiver::new(self.clone(), receiver);
        return Channel::new(sender, receiver);
    }

    fn spawn_event_handler<U: EventHandlerTrait>(&self, thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<U::Event>>, event_handler: U, join_call_back: impl AsyncJoinCallBackTrait<Self, U::ThreadReturn>) -> std::io::Result<EventHandlerSender<Self, U::Event>> {
        let (thread_builder, channel) = thread_builder.take();
        let (sender, receiver) = channel.take();

        let thread = EventHandlerThread::new(
            receiver,
            event_handler
        );

        thread_builder.spawn_thread(thread, join_call_back)?;

        return Ok(sender);
    }

    fn spawn_tcp_listener<T: TcpConnectionHandlerTrait<Factory=Self>>(&self, thread_builder: channel::ChannelThreadBuilder<Self, EventOrStopThread<()>>, socket_addr: SocketAddr, tcp_connection_handler: T, join_call_back: impl AsyncJoinCallBackTrait<Self, T>) -> Result<EventHandlerSender<Self, ()>, Error> {
        let tcp_listener = TcpListener::bind(socket_addr)?;
        let event_handler = TcpListenerEventHandler::new(tcp_listener, tcp_connection_handler);
        return thread_builder.spawn_event_handler(event_handler, join_call_back);
    }

    fn connect_tcp(&self, socket_addr: SocketAddr) -> Result<(Self::TcpWriter, Self::TcpReader), Error> {
        let tcp_stream = TcpStream::connect(socket_addr.clone())?;
        let real_tcp_stream = RealTcpStream::new(tcp_stream, socket_addr);
        return Ok((real_tcp_stream.try_clone()?, real_tcp_stream));
    }

    fn spawn_tcp_reader<T: TcpReadHandlerTrait>(&self, thread_builder: channel::ChannelThreadBuilder<Self, EventOrStopThread<()>>, tcp_reader: Self::TcpReader, tcp_read_handler: T, join_call_back: impl AsyncJoinCallBackTrait<Self, T>) -> Result<EventHandlerSender<Self, ()>, Error> {
        let event_handler = TcpReaderEventHandler::new(tcp_reader, tcp_read_handler);
        return thread_builder.spawn_event_handler(event_handler, join_call_back);
    }

    fn bind_udp_socket(&self, socket_addr: SocketAddr) -> Result<Self::UdpSocket, Error> {
        return RealUdpSocket::bind(socket_addr);
    }

    fn spawn_udp_reader<T: UdpReadHandlerTrait>(&self, thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<()>>, udp_socket: Self::UdpSocket, udp_read_handler: T, join_call_back: impl AsyncJoinCallBackTrait<Self, T>) -> Result<EventHandlerSender<Self, ()>, Error> {
        let event_handler = UdpReaderEventHandler::new(udp_socket, udp_read_handler);
        return thread_builder.spawn_event_handler(event_handler, join_call_back);
    }
}
