use crate::factory::FactoryTrait;
use crate::net::{TcpConnectionHandlerTrait, TcpReadHandlerTrait, UdpReadHandlerTrait};
use crate::threading::asyncjoin::AsyncJoin;
use crate::threading::eventhandling::{EventHandlerSender, EventHandlerTrait, EventOrStopThread};
use crate::threading::Thread;
use crate::threading::{channel, AsyncJoinCallBackTrait};
use log::info;
use std::io::Error;
use std::net::SocketAddr;
use std::thread::Builder;

pub struct ThreadBuilder<Factory: FactoryTrait> {
    factory: Factory,
    name: Option<String>,
}

impl<Factory: FactoryTrait> ThreadBuilder<Factory> {
    pub(crate) fn new(factory: Factory) -> Self {
        return Self {
            factory,
            name: None,
        };
    }

    pub fn get_factory(&self) -> &Factory {
        return &self.factory;
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

    pub fn build_channel_thread<T: Send + 'static>(
        self,
    ) -> channel::ChannelThreadBuilder<Factory, T> {
        return channel::ChannelThreadBuilder::new(self);
    }

    pub fn build_channel_for_event_handler<T: EventHandlerTrait>(
        self,
    ) -> channel::ChannelThreadBuilder<Factory, EventOrStopThread<T::Event>> {
        return self.build_channel_thread();
    }

    pub fn spawn_event_handler<T: EventHandlerTrait>(
        self,
        event_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Factory, T::ThreadReturn>,
    ) -> Result<EventHandlerSender<Factory, T::Event>, Error> {
        return self
            .build_channel_for_event_handler::<T>()
            .spawn_event_handler(event_handler, join_call_back);
    }

    pub fn spawn_tcp_listener<T: TcpConnectionHandlerTrait<Factory = Factory>>(
        self,
        socket_addr: SocketAddr,
        tcp_connection_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Factory, T>,
    ) -> Result<EventHandlerSender<Factory, ()>, Error> {
        return self
            .build_channel_thread::<EventOrStopThread<()>>()
            .spawn_tcp_listener(socket_addr, tcp_connection_handler, join_call_back);
    }

    pub fn spawn_tcp_reader<T: TcpReadHandlerTrait>(
        self,
        tcp_reader: Factory::TcpReader,
        tcp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Factory, T>,
    ) -> Result<EventHandlerSender<Factory, ()>, Error> {
        return self
            .build_channel_thread::<EventOrStopThread<()>>()
            .spawn_tcp_reader(tcp_reader, tcp_read_handler, join_call_back);
    }

    pub fn spawn_udp_reader<T: UdpReadHandlerTrait>(
        self,
        udp_socket: Factory::UdpSocket,
        udp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Factory, T>,
    ) -> Result<EventHandlerSender<Factory, ()>, Error> {
        return self
            .build_channel_thread::<EventOrStopThread<()>>()
            .spawn_udp_reader(udp_socket, udp_read_handler, join_call_back);
    }

    pub(crate) fn spawn_thread<T: Thread>(
        self,
        thread: T,
        join_call_back: impl AsyncJoinCallBackTrait<Factory, T::ReturnType>,
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
