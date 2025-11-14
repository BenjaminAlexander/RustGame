use crate::net::{
    TcpConnectionHandlerTrait,
    TcpReader,
    TcpStream,
};
use crate::real_time::{EventHandleResult, EventHandlerTrait, ReceiveMetaData};
use crate::single_threaded_simulator::net::ChannelTcpWriter;
use crate::single_threaded_simulator::SingleThreadedReceiver;
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
        &mut self,
        writer: ChannelTcpWriter,
        reader: SingleThreadedReceiver<Vec<u8>>,
    ) -> EventHandleResult<Self> {
        return match self.connection_handler.on_connection(
            TcpStream::new_simulated(writer),
            TcpReader::new_simulated(reader),
        ) {
            Continue(()) => EventHandleResult::TryForNextEvent,
            Break(()) => EventHandleResult::StopThread(()),
        };
    }
}

impl<TcpConnectionHandler: TcpConnectionHandlerTrait> EventHandlerTrait
    for TcpListenerEventHandler<TcpConnectionHandler>
{
    type Event = TcpListenerEvent;
    type ThreadReturn = ();   

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        return ();
    }
    
    fn on_event(&mut self, _: ReceiveMetaData, event: Self::Event) -> EventHandleResult<Self> {
        match event {
            TcpListenerEvent::ListenerReady => {
                self.connection_handler.on_bind(self.socket_addr);
                EventHandleResult::TryForNextEvent
            },
            TcpListenerEvent::Connection(writer, reader) => self.on_connection(writer, reader),
        }
    }

    fn on_channel_disconnect(&mut self) -> EventHandleResult<Self> {
        EventHandleResult::StopThread(())
    }
}
