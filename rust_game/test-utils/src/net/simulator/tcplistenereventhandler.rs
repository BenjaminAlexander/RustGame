use std::ops::ControlFlow::{Break, Continue};
use commons::net::TcpConnectionHandlerTrait;
use commons::threading::channel::ReceiveMetaData;
use commons::threading::eventhandling::{ChannelEvent, ChannelEventResult, EventHandlerTrait};
use commons::threading::eventhandling::WaitOrTryForNextEvent::{TryForNextEvent, WaitForNextEvent};
use crate::net::ChannelTcpWriter;
use crate::net::simulator::tcplistenereventhandler::TcpListenerEvent::Connection;
use crate::singlethreaded::{SingleThreadedFactory, SingleThreadedReceiver};

pub enum TcpListenerEvent {
    Connection(ChannelTcpWriter, SingleThreadedReceiver<Vec<u8>>)
}

pub struct TcpListenerEventHandler<TcpConnectionHandler: TcpConnectionHandlerTrait<Factory=SingleThreadedFactory>> {
    connection_handler: TcpConnectionHandler
}

impl<TcpConnectionHandler: TcpConnectionHandlerTrait<Factory=SingleThreadedFactory>> TcpListenerEventHandler<TcpConnectionHandler> {

    pub fn new(connection_handler: TcpConnectionHandler) -> Self {
        return Self {
            connection_handler
        }
    }

    fn on_connection(mut self, writer: ChannelTcpWriter, reader: SingleThreadedReceiver<Vec<u8>>) -> ChannelEventResult<Self> {
        return match self.connection_handler.on_connection(writer, reader) {
            Continue(()) => Continue(TryForNextEvent(self)),
            Break(()) => Break(self.connection_handler)
        };
    }
}

impl<TcpConnectionHandler: TcpConnectionHandlerTrait<Factory=SingleThreadedFactory>> EventHandlerTrait for TcpListenerEventHandler<TcpConnectionHandler> {
    type Event = TcpListenerEvent;
    type ThreadReturn = TcpConnectionHandler;

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> ChannelEventResult<Self> {
        return match channel_event {
            ChannelEvent::ReceivedEvent(_, Connection(writer, reader)) => self.on_connection(writer, reader),
            ChannelEvent::Timeout => Continue(TryForNextEvent(self)),
            ChannelEvent::ChannelEmpty => Continue(WaitForNextEvent(self)),
            ChannelEvent::ChannelDisconnected => Break(self.connection_handler)
        };
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        return self.connection_handler;
    }
}