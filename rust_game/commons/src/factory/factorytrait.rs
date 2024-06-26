use crate::net::{
    TcpConnectionHandlerTrait,
    TcpReadHandlerTrait,
    TcpWriterTrait,
    UdpReadHandlerTrait,
    UdpSocketTrait,
    LOCAL_EPHEMERAL_SOCKET_ADDR_V4,
};
use crate::threading::channel::{
    Channel,
    ChannelThreadBuilder,
    ReceiverTrait,
    SenderTrait,
};
use crate::threading::eventhandling::{
    EventHandlerSender,
    EventHandlerTrait,
    EventOrStopThread,
};
use crate::threading::{
    AsyncJoinCallBackTrait,
    ThreadBuilder,
};
use crate::time::TimeValue;
use std::io::Error;
use std::net::SocketAddr;

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

    fn spawn_event_handler<U: EventHandlerTrait>(
        &self,
        thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<U::Event>>,
        event_handler: U,
        join_call_back: impl AsyncJoinCallBackTrait<Self, U::ThreadReturn>,
    ) -> Result<EventHandlerSender<Self, U::Event>, Error>;

    fn spawn_tcp_listener<T: TcpConnectionHandlerTrait<Self>>(
        &self,
        thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<()>>,
        socket_addr: SocketAddr,
        tcp_connection_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Self, T>,
    ) -> Result<EventHandlerSender<Self, ()>, Error>;

    fn connect_tcp(
        &self,
        socket_addr: SocketAddr,
    ) -> Result<(Self::TcpWriter, Self::TcpReader), Error>;

    fn spawn_tcp_reader<T: TcpReadHandlerTrait>(
        &self,
        thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<()>>,
        tcp_reader: Self::TcpReader,
        tcp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Self, T>,
    ) -> Result<EventHandlerSender<Self, ()>, Error>;

    fn bind_udp_socket(&self, socket_addr: SocketAddr) -> Result<Self::UdpSocket, Error>;

    fn bind_udp_ephemeral_port(&self) -> Result<Self::UdpSocket, Error> {
        return self.bind_udp_socket(SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4));
    }

    fn spawn_udp_reader<T: UdpReadHandlerTrait>(
        &self,
        thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<()>>,
        udp_socket: Self::UdpSocket,
        udp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Self, T>,
    ) -> Result<EventHandlerSender<Self, ()>, Error>;
}
