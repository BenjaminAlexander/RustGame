use crate::net::simulator::tcplistenereventhandler::TcpListenerEvent::Connection;
use crate::net::ChannelTcpWriter;
use crate::singlethreaded::{
    SingleThreadedFactory,
    SingleThreadedReceiver,
};
use commons::net::TcpConnectionHandlerTrait;
use commons::threading::channel::ReceiveMetaData;
use commons::threading::eventhandling::{
    ChannelEvent,
    EventHandleResult,
    EventHandlerTrait,
};
use std::ops::ControlFlow::{
    Break,
    Continue,
};

pub enum TcpListenerEvent {
    Connection(ChannelTcpWriter, SingleThreadedReceiver<Vec<u8>>),
}

pub struct TcpListenerEventHandler<
    TcpConnectionHandler: TcpConnectionHandlerTrait<SingleThreadedFactory>,
> {
    connection_handler: TcpConnectionHandler,
}

impl<TcpConnectionHandler: TcpConnectionHandlerTrait<SingleThreadedFactory>>
    TcpListenerEventHandler<TcpConnectionHandler>
{
    pub fn new(connection_handler: TcpConnectionHandler) -> Self {
        return Self { connection_handler };
    }

    fn on_connection(
        mut self,
        writer: ChannelTcpWriter,
        reader: SingleThreadedReceiver<Vec<u8>>,
    ) -> EventHandleResult<Self> {
        return match self.connection_handler.on_connection(writer, reader) {
            Continue(()) => EventHandleResult::TryForNextEvent(self),
            Break(()) => EventHandleResult::StopThread(self.connection_handler),
        };
    }
}

impl<TcpConnectionHandler: TcpConnectionHandlerTrait<SingleThreadedFactory>>
    EventHandlerTrait for TcpListenerEventHandler<TcpConnectionHandler>
{
    type Event = TcpListenerEvent;
    type ThreadReturn = TcpConnectionHandler;

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> EventHandleResult<Self> {
        return match channel_event {
            ChannelEvent::ReceivedEvent(_, Connection(writer, reader)) => {
                self.on_connection(writer, reader)
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
