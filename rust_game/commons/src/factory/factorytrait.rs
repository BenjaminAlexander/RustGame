use std::io::Error;
use std::net::SocketAddr;
use crate::net::{TcpConnectionHandlerTrait, TcpReaderTrait, TcpReadHandlerTrait, TcpWriterTrait};
use crate::threading::channel::{Channel, SenderTrait};
use crate::threading::{AsyncJoinCallBackTrait, channel, eventhandling, ThreadBuilder};
use crate::threading::eventhandling::{EventHandlerTrait, EventOrStopThread};
use crate::time::TimeValue;

pub trait FactoryTrait: Clone + Send + 'static {
    type Sender<T: Send>: SenderTrait<T>;
    type TcpWriter: TcpWriterTrait;
    type TcpReader: TcpReaderTrait;

    fn now(&self) -> TimeValue;

    fn new_thread_builder(&self) -> ThreadBuilder<Self> {
        return ThreadBuilder::new(self.clone());
    }

    fn new_channel<T: Send>(&self) -> Channel<Self, T>;

    //TODO: make this less args
    //TODO: call from channel thread builder
    fn spawn_event_handler<T: Send, U: EventHandlerTrait<Event=T>>(
        &self,
        thread_builder: ThreadBuilder<Self>,
        channel: Channel<Self, EventOrStopThread<T>>,
        event_handler: U,
        join_call_back: impl AsyncJoinCallBackTrait<Self, U::ThreadReturn>
    ) -> Result<eventhandling::Sender<Self, T>, Error>;

    fn spawn_tcp_listener<T: TcpConnectionHandlerTrait<Factory=Self>>(
        &self,
        thread_builder: channel::ThreadBuilder<Self, EventOrStopThread<()>>,
        socket_addr: SocketAddr,
        tcp_connection_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Self, T>
    ) -> Result<eventhandling::Sender<Self, ()>, Error>;

    fn connect_tcp(&self, socket_addr: SocketAddr) -> Result<(Self::TcpWriter, Self::TcpReader), Error>;

    fn spawn_tcp_reader<T: TcpReadHandlerTrait>(
        &self,
        thread_builder: channel::ThreadBuilder<Self, EventOrStopThread<()>>,
        tcp_reader: Self::TcpReader,
        read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Self, T>
    ) -> Result<eventhandling::Sender<Self, ()>, Error>;
}