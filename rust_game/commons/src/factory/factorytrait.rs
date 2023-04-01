use std::io::Error;
use std::net::SocketAddr;
use crate::net::{TcpConnectionHandlerTrait, TcpReaderTrait, TcpReadHandlerTrait, TcpSenderTrait};
use crate::threading::channel::{Channel, SenderTrait};
use crate::threading::{AsyncJoinCallBackTrait, eventhandling, ThreadBuilder};
use crate::threading::eventhandling::{EventHandlerTrait, EventOrStopThread};
use crate::time::TimeValue;

pub trait FactoryTrait: Clone + Send + 'static {
    type Sender<T: Send>: SenderTrait<T>;

    //TODO: rename as writer
    type TcpSender: TcpSenderTrait;

    //TODO: rename as reader
    type TcpReceiver: TcpReaderTrait;

    fn now(&self) -> TimeValue;

    fn new_thread_builder(&self) -> ThreadBuilder<Self> {
        return ThreadBuilder::new(self.clone());
    }

    fn new_channel<T: Send>(&self) -> Channel<Self, T>;

    //TODO: make this less args
    fn spawn_event_handler<T: Send, U: EventHandlerTrait<Event=T>>(
        &self,
        thread_builder: ThreadBuilder<Self>,
        channel: Channel<Self, EventOrStopThread<T>>,
        event_handler: U,
        join_call_back: impl AsyncJoinCallBackTrait<Self, U::ThreadReturn>
    ) -> Result<eventhandling::Sender<Self, T>, Error>;

    fn spawn_tcp_listener<T: TcpConnectionHandlerTrait<TcpSender=Self::TcpSender, TcpReceiver=Self::TcpReceiver>>(
        &self,
        socket_addr: SocketAddr,
        tcp_connection_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Self, T>
    ) -> Result<eventhandling::Sender<Self, ()>, Error>;

    fn connect_tcp(&self, socket_addr: SocketAddr) -> Result<(Self::TcpSender, Self::TcpReceiver), Error>;

    fn spawn_tcp_reader<T: TcpReadHandlerTrait>(
        &self,
        tcp_reader: Self::TcpReceiver,
        read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Self, T>
    ) -> Result<eventhandling::Sender<Self, ()>, Error>;
}