use crate::factory::FactoryTrait;
use crate::net::{
    TcpConnectionHandlerTrait,
    TcpReadHandlerTrait,
    UdpReadHandlerTrait,
};
use crate::threading;
use crate::threading::channel::Channel;
use crate::threading::eventhandling::{
    EventHandlerSender,
    EventHandlerTrait,
    EventOrStopThread,
};
use crate::threading::AsyncJoinCallBackTrait;
use std::io::Error;
use std::net::SocketAddr;

pub struct ChannelThreadBuilder<Factory: FactoryTrait, T: Send + 'static> {
    factory: Factory,
    thread_builder: threading::ThreadBuilder,
    channel: Channel<Factory, T>,
}

impl<Factory: FactoryTrait, T: Send + 'static> ChannelThreadBuilder<Factory, T> {
    pub fn new(factory: Factory, thread_builder: threading::ThreadBuilder) -> Self {
        let channel = factory.new_channel();
        return Self {
            factory,
            channel,
            thread_builder,
        };
    }

    pub fn get_channel(&self) -> &Channel<Factory, T> {
        return &self.channel;
    }

    pub fn get_sender(&self) -> &Factory::Sender<T> {
        return self.get_channel().get_sender();
    }

    //TODO: maybe remove this guy
    pub fn clone_sender(&self) -> Factory::Sender<T> {
        return (*self.get_channel().get_sender()).clone();
    }

    pub fn take(self) -> (threading::ThreadBuilder, Channel<Factory, T>) {
        return (self.thread_builder, self.channel);
    }
}

impl<Factory: FactoryTrait, T: Send + 'static> ChannelThreadBuilder<Factory, EventOrStopThread<T>> {
    pub fn spawn_event_handler<U: EventHandlerTrait<Event = T>>(
        self,
        event_handler: U,
        join_call_back: impl AsyncJoinCallBackTrait<U::ThreadReturn>,
    ) -> Result<EventHandlerSender<Factory, T>, Error> {
        let factory = self.factory.clone();
        return factory.spawn_event_handler(self, event_handler, join_call_back);
    }
}

impl<Factory: FactoryTrait> ChannelThreadBuilder<Factory, EventOrStopThread<()>> {
    /* 
    pub fn spawn_tcp_listener<T: TcpConnectionHandlerTrait<Factory>>(
        self,
        socket_addr: SocketAddr,
        tcp_connection_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T>,
    ) -> Result<EventHandlerSender<Factory, ()>, Error> {
        let factory = self.factory.clone();
        return factory.spawn_tcp_listener(
            self,
            socket_addr,
            tcp_connection_handler,
            join_call_back,
        );
    }
    */

    pub fn spawn_tcp_reader<T: TcpReadHandlerTrait>(
        self,
        tcp_reader: Factory::TcpReader,
        tcp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T>,
    ) -> Result<EventHandlerSender<Factory, ()>, Error> {
        let factory = self.factory.clone();
        return factory.spawn_tcp_reader(self, tcp_reader, tcp_read_handler, join_call_back);
    }

    pub fn spawn_udp_reader<T: UdpReadHandlerTrait>(
        self,
        udp_socket: Factory::UdpSocket,
        udp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T>,
    ) -> Result<EventHandlerSender<Factory, ()>, Error> {
        let factory = self.factory.clone();
        return factory.spawn_udp_reader(self, udp_socket, udp_read_handler, join_call_back);
    }
}
