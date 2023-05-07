use std::io::Error;
use std::net::SocketAddr;
use crate::net::{TcpConnectionHandlerTrait, TcpReadHandlerTrait, TcpWriterTrait, UdpReadHandlerTrait, UdpSocketTrait};
use crate::threading::channel::{Channel, ChannelThreadBuilder, ReceiverTrait, SenderTrait};
use crate::threading::{AsyncJoinCallBackTrait, ThreadBuilder};
use crate::threading::eventhandling::{EventHandlerTrait, EventOrStopThread, EventSender};
use crate::time::TimeValue;

pub trait FactoryTrait: Clone + Send + 'static {
    type Sender<T: Send>: SenderTrait<T>;
    type Receiver<T: Send>: ReceiverTrait<T>;

    type TcpWriter: TcpWriterTrait;
    type TcpReader: Send + Sized;

    type UdpSocket: UdpSocketTrait;

    fn now(&self) -> TimeValue;

    fn new_thread_builder(&self) -> ThreadBuilder<Self> {
        return ThreadBuilder::new(self.clone());
    }

    fn new_channel<T: Send>(&self) -> Channel<Self, T>;

    //TODO: make this less args
    //TODO: remove T type arg
    //TODO: call from channel thread builder
    fn spawn_event_handler<U: EventHandlerTrait>(
        &self,
        thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<U::Event>>,
        event_handler: U,
        join_call_back: impl AsyncJoinCallBackTrait<Self, U::ThreadReturn>
    ) -> Result<EventSender<Self, U::Event>, Error>;

    fn spawn_tcp_listener<T: TcpConnectionHandlerTrait<Factory=Self>>(
        &self,
        thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<()>>,
        socket_addr: SocketAddr,
        tcp_connection_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Self, T>
    ) -> Result<EventSender<Self, ()>, Error>;

    fn connect_tcp(&self, socket_addr: SocketAddr) -> Result<(Self::TcpWriter, Self::TcpReader), Error>;

    fn spawn_tcp_reader<T: TcpReadHandlerTrait>(
        &self,
        thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<()>>,
        tcp_reader: Self::TcpReader,
        tcp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Self, T>
    ) -> Result<EventSender<Self, ()>, Error>;

    fn bind_udp_socket(&self, socket_addr: SocketAddr) -> Result<Self::UdpSocket, Error>;

    fn spawn_udp_reader<T: UdpReadHandlerTrait>(
        &self,
        thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<()>>,
        udp_socket: Self::UdpSocket,
        udp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Self, T>
    ) -> Result<EventSender<Self, ()>, Error>;
}
