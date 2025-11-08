use crate::factory::FactoryTrait;
use crate::net::{
    TcpConnectionHandlerTrait,
    TcpReadHandlerTrait,
    TcpReader,
    UdpReadHandlerTrait,
    UdpSocket,
};
use crate::threading::asyncjoin::AsyncJoin;
use crate::threading::eventhandling::{
    EventHandlerSender,
    EventHandlerTrait,
    EventOrStopThread,
};
use crate::threading::Thread;
use crate::threading::{
    channel,
    AsyncJoinCallBackTrait,
};
use log::info;
use std::io::Error;
use std::net::SocketAddr;
use std::thread::Builder;

pub struct ThreadBuilder {}

impl ThreadBuilder {
    pub(crate) fn new() -> Self {
        return Self {};
    }

    pub fn build_channel_thread<T: Send + 'static>(
        factory: &impl FactoryTrait,
    ) -> channel::ChannelThreadBuilder<T> {
        let channel = factory.new_channel();
        return channel::ChannelThreadBuilder::new(channel);
    }

    pub fn build_channel_for_event_handler<T: EventHandlerTrait>(
        factory: &impl FactoryTrait,
    ) -> channel::ChannelThreadBuilder<EventOrStopThread<T::Event>> {
        return Self::build_channel_thread(factory);
    }

    pub fn build_channel_for_tcp_listener<T: TcpConnectionHandlerTrait>(
        factory: &impl FactoryTrait,
    ) -> channel::ChannelThreadBuilder<EventOrStopThread<()>> {
        return Self::build_channel_thread(factory);
    }

    pub fn spawn_event_handler<T: EventHandlerTrait>(
        factory: &impl FactoryTrait,
        thread_name: String,
        event_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T::ThreadReturn>,
    ) -> Result<EventHandlerSender<T::Event>, Error> {
        let thread_builder = Self::build_channel_for_event_handler::<T>(factory);

        return factory.spawn_event_handler(
            thread_name,
            thread_builder,
            event_handler,
            join_call_back,
        );
    }

    pub fn spawn_tcp_listener<T: TcpConnectionHandlerTrait>(
        factory: &impl FactoryTrait,
        thread_name: String,
        socket_addr: SocketAddr,
        tcp_connection_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T>,
    ) -> Result<EventHandlerSender<()>, Error> {
        let thread_builder = Self::build_channel_thread::<EventOrStopThread<()>>(factory);

        return factory.spawn_tcp_listener(
            thread_name,
            thread_builder,
            socket_addr,
            tcp_connection_handler,
            join_call_back,
        );
    }

    pub fn spawn_tcp_reader<T: TcpReadHandlerTrait>(
        factory: &impl FactoryTrait,
        thread_name: String,
        tcp_reader: TcpReader,
        tcp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T>,
    ) -> Result<EventHandlerSender<()>, Error> {
        let thread_builder: channel::ChannelThreadBuilder<EventOrStopThread<()>> =
            Self::build_channel_thread::<EventOrStopThread<()>>(factory);

        return factory.spawn_tcp_reader(
            thread_name,
            thread_builder,
            tcp_reader,
            tcp_read_handler,
            join_call_back,
        );
    }

    pub fn spawn_udp_reader<T: UdpReadHandlerTrait>(
        factory: &impl FactoryTrait,
        thread_name: String,
        udp_socket: UdpSocket,
        udp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T>,
    ) -> Result<EventHandlerSender<()>, Error> {
        let thread_builder = Self::build_channel_thread::<EventOrStopThread<()>>(factory);

        return factory.spawn_udp_reader(
            thread_name,
            thread_builder,
            udp_socket,
            udp_read_handler,
            join_call_back,
        );
    }

    pub(crate) fn spawn_thread<T: Thread>(
        thread_name: String,
        thread: T,
        join_call_back: impl AsyncJoinCallBackTrait<T::ReturnType>,
    ) -> std::io::Result<()> {
        let builder = Builder::new().name(thread_name.clone());

        builder.spawn(|| {
            info!("Thread Starting");

            let return_value = thread.run();
            let async_join = AsyncJoin::new(thread_name, return_value);
            join_call_back.join(async_join);

            info!("Thread Ending");
        })?;

        return Ok(());
    }
}
