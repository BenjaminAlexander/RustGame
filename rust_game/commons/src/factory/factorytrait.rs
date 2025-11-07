use crate::net::{
    TcpConnectionHandlerTrait,
    TcpReadHandlerTrait,
    TcpReader,
    TcpStream,
    UdpReadHandlerTrait,
    UdpSocket,
    LOCAL_EPHEMERAL_SOCKET_ADDR_V4,
};
use crate::threading::channel::{
    Channel,
    ChannelThreadBuilder,
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
use crate::time::TimeSource;
use std::io::Error;
use std::net::SocketAddr;

pub trait FactoryTrait: Clone + Send + 'static {
    fn get_time_source(&self) -> &TimeSource;

    fn new_channel<T: Send>(&self) -> Channel<T>;

    fn connect_tcp(&self, socket_addr: SocketAddr) -> Result<(TcpStream, TcpReader), Error>;

    fn bind_udp_socket(&self, socket_addr: SocketAddr) -> Result<UdpSocket, Error>;

    //TODO: remove this, or add it back as a way to propogate the factory
    fn new_thread_builder(&self) -> ThreadBuilder {
        return ThreadBuilder::new();
    }

    fn spawn_event_handler<U: EventHandlerTrait>(
        &self,
        thread_builder: ChannelThreadBuilder<EventOrStopThread<U::Event>>,
        event_handler: U,
        join_call_back: impl AsyncJoinCallBackTrait<U::ThreadReturn>,
    ) -> std::io::Result<EventHandlerSender<U::Event>> {
        let (thread_builder, channel) = thread_builder.take();
        let (sender, receiver) = channel.take();
        receiver.spawn_event_handler(thread_builder, event_handler, join_call_back)?;
        return Ok(sender);
    }

    fn spawn_tcp_listener<T: TcpConnectionHandlerTrait>(
        &self,
        thread_builder: ChannelThreadBuilder<EventOrStopThread<()>>,
        socket_addr: SocketAddr,
        tcp_connection_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T>,
    ) -> Result<EventHandlerSender<()>, Error> {
        let (thread_builder, channel) = thread_builder.take();
        let (sender, receiver) = channel.take();
        receiver.spawn_tcp_listener(
            thread_builder,
            socket_addr,
            tcp_connection_handler,
            join_call_back,
        )?;
        return Ok(sender);
    }

    fn spawn_tcp_reader<T: TcpReadHandlerTrait>(
        &self,
        thread_builder: ChannelThreadBuilder<EventOrStopThread<()>>,
        tcp_reader: TcpReader,
        tcp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T>,
    ) -> Result<EventHandlerSender<()>, Error> {
        let (thread_builder, channel) = thread_builder.take();
        let (sender, receiver) = channel.take();
        tcp_reader.spawn_tcp_reader(thread_builder, receiver, tcp_read_handler, join_call_back)?;
        return Ok(sender);
    }

    fn bind_udp_ephemeral_port(&self) -> Result<UdpSocket, Error> {
        return self.bind_udp_socket(SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4));
    }

    fn spawn_udp_reader<T: UdpReadHandlerTrait>(
        &self,
        thread_builder: ChannelThreadBuilder<EventOrStopThread<()>>,
        udp_socket: UdpSocket,
        udp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T>,
    ) -> Result<EventHandlerSender<()>, Error> {
        let (thread_builder, channel) = thread_builder.take();
        let (sender, receiver) = channel.take();
        udp_socket.spawn_udp_reader(thread_builder, receiver, udp_read_handler, join_call_back)?;
        return Ok(sender);
    }
}
