use crate::factory::FactoryTrait;
use crate::net::{
    TcpConnectionHandlerTrait,
    TcpReadHandlerTrait,
    UdpReadHandlerTrait,
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

pub struct ThreadBuilder {
    name: Option<String>,
}

impl ThreadBuilder {
    pub(crate) fn new() -> Self {
        return Self {
            name: None,
        };
    }

    pub fn name(self, name: &str) -> Self {
        return self.set_name_from_string(name.to_string());
    }

    pub fn set_name_from_string(mut self, name: String) -> Self {
        self.name = Some(name);
        return self;
    }

    pub fn get_name(&self) -> Option<&String> {
        return self.name.as_ref();
    }

    pub fn build_channel_thread<Factory: FactoryTrait, T: Send + 'static>(
        self,
        factory: Factory
    ) -> channel::ChannelThreadBuilder<Factory, T> {
        return channel::ChannelThreadBuilder::new(factory, self);
    }

    pub fn build_channel_for_event_handler<Factory: FactoryTrait, T: EventHandlerTrait>(
        self,
        factory: Factory
    ) -> channel::ChannelThreadBuilder<Factory, EventOrStopThread<T::Event>> {
        return self.build_channel_thread(factory);
    }

    pub fn build_channel_for_tcp_listener<Factory: FactoryTrait, T: TcpConnectionHandlerTrait<Factory>>(
        self,
        factory: Factory
    ) -> channel::ChannelThreadBuilder<Factory, EventOrStopThread<()>> {
        return self.build_channel_thread(factory);
    }

    pub fn spawn_event_handler<Factory: FactoryTrait, T: EventHandlerTrait>(
        self,
        factory: Factory,
        event_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T::ThreadReturn>,
    ) -> Result<EventHandlerSender<Factory, T::Event>, Error> {
        return self
            .build_channel_for_event_handler::<Factory, T>(factory)
            .spawn_event_handler(event_handler, join_call_back);
    }

    pub fn spawn_tcp_listener<Factory: FactoryTrait, T: TcpConnectionHandlerTrait<Factory>>(
        self,
        factory: Factory,
        socket_addr: SocketAddr,
        tcp_connection_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T>,
    ) -> Result<EventHandlerSender<Factory, ()>, Error> {

        let thread_builder = self.build_channel_thread::<Factory, EventOrStopThread<()>>(factory.clone());

        return factory.spawn_tcp_listener(
            thread_builder,
            socket_addr,
            tcp_connection_handler,
            join_call_back,
        );
    }

    pub fn spawn_tcp_reader<Factory: FactoryTrait, T: TcpReadHandlerTrait>(
        self,
        factory: Factory,
        tcp_reader: Factory::TcpReader,
        tcp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T>,
    ) -> Result<EventHandlerSender<Factory, ()>, Error> {
        return self
            .build_channel_thread::<Factory, EventOrStopThread<()>>(factory)
            .spawn_tcp_reader(tcp_reader, tcp_read_handler, join_call_back);
    }

    pub fn spawn_udp_reader<Factory: FactoryTrait, T: UdpReadHandlerTrait>(
        self,
        factory: Factory,
        udp_socket: Factory::UdpSocket,
        udp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T>,
    ) -> Result<EventHandlerSender<Factory, ()>, Error> {
        return self
            .build_channel_thread::<Factory, EventOrStopThread<()>>(factory)
            .spawn_udp_reader(udp_socket, udp_read_handler, join_call_back);
    }

    pub(crate) fn spawn_thread<T: Thread>(
        self,
        thread: T,
        join_call_back: impl AsyncJoinCallBackTrait<T::ReturnType>,
    ) -> std::io::Result<()> {
        let mut builder = Builder::new();

        if let Some(name) = self.name.as_ref() {
            builder = builder.name(name.clone());
        }

        builder.spawn(|| {
            info!("Thread Starting");

            let return_value = thread.run();
            let async_join = AsyncJoin::new(self, return_value);
            join_call_back.join(async_join);

            info!("Thread Ending");
        })?;

        return Ok(());
    }
}
