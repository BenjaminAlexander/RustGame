use std::io::Error;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::factory::FactoryTrait;
use crate::net::{RealTcpStream, TcpConnectionHandlerTrait, TcpListenerEventHandler, TcpReaderEventHandler, TcpReadHandlerTrait};
use crate::threading::channel::{Channel, ChannelThreadBuilder, RealSender, RealReceiver, SendMetaData};
use crate::threading::eventhandling::{EventHandlerThread, EventHandlerTrait, EventOrStopThread, Sender};
use crate::threading::{AsyncJoinCallBackTrait, channel};
use crate::time::TimeValue;

#[derive(Clone, Copy)]
pub struct RealFactory {

}

impl RealFactory {
    pub fn new() -> Self {
        return Self {};
    }
}

impl FactoryTrait for RealFactory {
    type Sender<T: Send> = RealSender<Self, T>;
    type Receiver<T: Send> = RealReceiver<Self, T>;
    type TcpWriter = RealTcpStream;
    type TcpReader = RealTcpStream;

    fn now(&self) -> TimeValue {
        return TimeValue::from_seconds_since_epoch(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64());
    }

    fn new_channel<T: Send>(&self) -> Channel<Self, T> {
        let (sender, receiver) = mpsc::channel::<(SendMetaData, T)>();
        let sender = RealSender::new(self.clone(), sender);
        let receiver = RealReceiver::new(self.clone(), receiver);
        return Channel::new(sender, receiver);
    }

    fn spawn_event_handler<U: EventHandlerTrait>(&self, thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<U::Event>>, event_handler: U, join_call_back: impl AsyncJoinCallBackTrait<Self, U::ThreadReturn>) -> std::io::Result<Sender<Self, U::Event>> {
        let (thread_builder, channel) = thread_builder.take();
        let (sender, receiver) = channel.take();

        let thread = EventHandlerThread::new(
            receiver,
            event_handler
        );

        thread_builder.spawn_thread(thread, join_call_back)?;

        return Ok(sender);
    }

    fn spawn_tcp_listener<T: TcpConnectionHandlerTrait<Factory=Self>>(&self, thread_builder: channel::ChannelThreadBuilder<Self, EventOrStopThread<()>>, socket_addr: SocketAddr, tcp_connection_handler: T, join_call_back: impl AsyncJoinCallBackTrait<Self, T>) -> Result<Sender<Self, ()>, Error> {
        let tcp_listener = TcpListener::bind(socket_addr)?;
        let event_handler = TcpListenerEventHandler::new(tcp_listener, tcp_connection_handler);
        return thread_builder.spawn_event_handler(event_handler, join_call_back);
    }

    fn connect_tcp(&self, socket_addr: SocketAddr) -> Result<(Self::TcpWriter, Self::TcpReader), Error> {
        let tcp_stream = TcpStream::connect(socket_addr.clone())?;
        let real_tcp_stream = RealTcpStream::new(tcp_stream, socket_addr);
        return Ok((real_tcp_stream.try_clone()?, real_tcp_stream));
    }

    fn spawn_tcp_reader<T: TcpReadHandlerTrait>(&self, thread_builder: channel::ChannelThreadBuilder<Self, EventOrStopThread<()>>, tcp_reader: Self::TcpReader, tcp_read_handler: T, join_call_back: impl AsyncJoinCallBackTrait<Self, T>) -> Result<Sender<Self, ()>, Error> {
        let event_handler = TcpReaderEventHandler::new(tcp_reader, tcp_read_handler);
        return thread_builder.spawn_event_handler(event_handler, join_call_back);
    }
}