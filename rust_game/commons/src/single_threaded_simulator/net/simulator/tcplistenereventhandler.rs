use crate::factory::FactoryTrait;
use crate::net::{
    TcpConnectionHandlerTrait,
    TcpStream,
};
use crate::single_threaded_simulator::net::ChannelTcpWriter;
use crate::single_threaded_simulator::{
    SingleThreadedReceiver,
};
use crate::threading::channel::ReceiveMetaData;
use crate::threading::eventhandling::{
    ChannelEvent,
    EventHandleResult,
    EventHandlerTrait,
};
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::ops::ControlFlow::{
    Break,
    Continue,
};

pub enum TcpListenerEvent {
    ListenerReady,
    Connection(ChannelTcpWriter, SingleThreadedReceiver<Vec<u8>>),
}

pub struct TcpListenerEventHandler<
    Factory: FactoryTrait,
    TcpConnectionHandler: TcpConnectionHandlerTrait<Factory>,
> {
    socket_addr: SocketAddr,
    connection_handler: TcpConnectionHandler,
    phantom: PhantomData<Factory>
}

impl<Factory: FactoryTrait, TcpConnectionHandler: TcpConnectionHandlerTrait<Factory>>
    TcpListenerEventHandler<Factory, TcpConnectionHandler>
{
    pub fn new(socket_addr: SocketAddr, connection_handler: TcpConnectionHandler) -> Self {
        return Self {
            socket_addr,
            connection_handler,
            phantom: PhantomData
        };
    }

    fn on_connection(
        mut self,
        writer: ChannelTcpWriter,
        reader: SingleThreadedReceiver<Vec<u8>>,
    ) -> EventHandleResult<Self> {
        return match self
            .connection_handler
            .on_connection(TcpStream::new_simulated(writer), reader)
        {
            Continue(()) => EventHandleResult::TryForNextEvent(self),
            Break(()) => EventHandleResult::StopThread(self.connection_handler),
        };
    }
}

impl<Factory: FactoryTrait, TcpConnectionHandler: TcpConnectionHandlerTrait<Factory>> EventHandlerTrait
    for TcpListenerEventHandler<Factory, TcpConnectionHandler>
{
    type Event = TcpListenerEvent;
    type ThreadReturn = TcpConnectionHandler;

    fn on_channel_event(
        mut self,
        channel_event: ChannelEvent<Self::Event>,
    ) -> EventHandleResult<Self> {
        return match channel_event {
            ChannelEvent::ReceivedEvent(_, TcpListenerEvent::Connection(writer, reader)) => {
                self.on_connection(writer, reader)
            }
            ChannelEvent::ReceivedEvent(_, TcpListenerEvent::ListenerReady) => {
                self.connection_handler.on_bind(self.socket_addr);
                EventHandleResult::TryForNextEvent(self)
            }
            ChannelEvent::Timeout => EventHandleResult::TryForNextEvent(self),
            ChannelEvent::ChannelEmpty => EventHandleResult::WaitForNextEvent(self),
            ChannelEvent::ChannelDisconnected => {
                EventHandleResult::StopThread(self.connection_handler)
            }
        };
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        return self.connection_handler;
    }
}
