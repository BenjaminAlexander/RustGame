use crate::real_time::net::tcp::{
    HandleTcpConnection,
    TcpReader,
    TcpStream,
};
use crate::real_time::simulation::net::tcp::ChannelTcpWriter;
use crate::real_time::simulation::SingleThreadedReceiver;
use crate::real_time::{
    EventHandleResult,
    HandleEvent,
    ReceiveMetaData,
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

pub struct TcpListenerEventHandler<TcpConnectionHandler: HandleTcpConnection> {
    socket_addr: SocketAddr,
    connection_handler: TcpConnectionHandler,
}

impl<TcpConnectionHandler: HandleTcpConnection> TcpListenerEventHandler<TcpConnectionHandler> {
    pub fn new(socket_addr: SocketAddr, connection_handler: TcpConnectionHandler) -> Self {
        return Self {
            socket_addr,
            connection_handler,
        };
    }

    fn on_connection(
        &mut self,
        writer: ChannelTcpWriter,
        reader: SingleThreadedReceiver<Vec<u8>>,
    ) -> EventHandleResult {
        return match self.connection_handler.on_connection(
            TcpStream::new_simulated(writer),
            TcpReader::new_simulated(reader),
        ) {
            Continue(()) => EventHandleResult::TryForNextEvent,
            Break(()) => EventHandleResult::StopThread,
        };
    }
}

impl<TcpConnectionHandler: HandleTcpConnection> HandleEvent
    for TcpListenerEventHandler<TcpConnectionHandler>
{
    type Event = TcpListenerEvent;
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, event: Self::Event) -> EventHandleResult {
        match event {
            TcpListenerEvent::ListenerReady => {
                self.connection_handler.on_bind(self.socket_addr);
                EventHandleResult::TryForNextEvent
            }
            TcpListenerEvent::Connection(writer, reader) => self.on_connection(writer, reader),
        }
    }

    fn on_stop_self(self) -> Self::ThreadReturn {
        return ();
    }
}
