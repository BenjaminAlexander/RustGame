use crate::net::{
    TcpConnectionHandlerTrait,
    TcpReader,
    TcpStream,
};
use crate::single_threaded_simulator::net::ChannelTcpWriter;
use crate::single_threaded_simulator::SingleThreadedReceiver;
use crate::threading::channel::ReceiveMetaData;
use crate::threading::eventhandling::{
    ChannelEvent,
    EventHandleResult,
    EventHandlerTrait,
};
use std::net::SocketAddr;
use std::ops::ControlFlow::{
    Break,
    Continue,
};

pub enum TcpListenerEvent {
    ListenerReady,
    Connection(ChannelTcpWriter, SingleThreadedReceiver<Vec<u8>>),
}

pub struct TcpListenerEventHandler<TcpConnectionHandler: TcpConnectionHandlerTrait> {
    socket_addr: SocketAddr,
    connection_handler: TcpConnectionHandler,
}

impl<TcpConnectionHandler: TcpConnectionHandlerTrait>
    TcpListenerEventHandler<TcpConnectionHandler>
{
    pub fn new(socket_addr: SocketAddr, connection_handler: TcpConnectionHandler) -> Self {
        return Self {
            socket_addr,
            connection_handler,
        };
    }

    fn on_connection(
        mut self,
        writer: ChannelTcpWriter,
        reader: SingleThreadedReceiver<Vec<u8>>,
    ) -> EventHandleResult<Self> {
        return match self.connection_handler.on_connection(
            TcpStream::new_simulated(writer),
            TcpReader::new_simulated(reader),
        ) {
            Continue(connection_handler) => {
                self.connection_handler = connection_handler;
                EventHandleResult::TryForNextEvent(self)
            }
            Break(()) => EventHandleResult::StopThread,
        };
    }
}

impl<TcpConnectionHandler: TcpConnectionHandlerTrait> EventHandlerTrait
    for TcpListenerEventHandler<TcpConnectionHandler>
{
    type Event = TcpListenerEvent;

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
                self.connection_handler.on_channel_disconnected();
                EventHandleResult::StopThread
            }
        };
    }

    fn on_stop(self, receive_meta_data: ReceiveMetaData) {
        self.connection_handler.on_stop(receive_meta_data);
    }
}
