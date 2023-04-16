use std::io::Error;
use std::net::SocketAddr;
use crate::net::{TcpConnectionHandlerTrait, TcpReadHandlerTrait, TcpWriterTrait};
use crate::threading::channel::{Channel, ChannelThreadBuilder, ReceiverTrait, SenderTrait};
use crate::threading::{AsyncJoinCallBackTrait, channel, eventhandling, ThreadBuilder};
use crate::threading::eventhandling::{EventHandlerTrait, EventOrStopThread};
use crate::time::TimeValue;

pub trait FactoryTrait: Clone + Send + 'static {
    type Sender<T: Send>: SenderTrait<T>;
    type Receiver<T: Send>: ReceiverTrait<T>;
    type TcpWriter: TcpWriterTrait;
    type TcpReader: Send + Sized;

    fn now(&self) -> TimeValue;

    fn new_thread_builder(&self) -> ThreadBuilder<Self> {
        return ThreadBuilder::new(self.clone());
    }

    fn new_channel<T: Send>(&self) -> Channel<Self, T>;

    //TODO: make this less args
    //TODO: remove T type arg
    //TODO: call from channel thread builder
    fn spawn_event_handler<U: EventHandlerTrait>(
        &self,
        thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<U::Event>>,
        event_handler: U,
        join_call_back: impl AsyncJoinCallBackTrait<Self, U::ThreadReturn>
    ) -> Result<eventhandling::Sender<Self, U::Event>, Error>;

    fn spawn_tcp_listener<T: TcpConnectionHandlerTrait<Factory=Self>>(
        &self,
        thread_builder: channel::ChannelThreadBuilder<Self, EventOrStopThread<()>>,
        socket_addr: SocketAddr,
        tcp_connection_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Self, T>
    ) -> Result<eventhandling::Sender<Self, ()>, Error>;

    fn connect_tcp(&self, socket_addr: SocketAddr) -> Result<(Self::TcpWriter, Self::TcpReader), Error>;

    fn spawn_tcp_reader<T: TcpReadHandlerTrait>(
        &self,
        thread_builder: channel::ChannelThreadBuilder<Self, EventOrStopThread<()>>,
        tcp_reader: Self::TcpReader,
        read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Self, T>
    ) -> Result<eventhandling::Sender<Self, ()>, Error>;
}