use std::io::Error;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::factory::FactoryTrait;
use crate::net::{RealTcpStream, TcpConnectionHandlerTrait, TcpListenerEventHandler, TcpReaderEventHandler, TcpReadHandlerTrait};
use crate::threading::channel::{Channel, RealSender, Receiver, SendMetaData};
use crate::threading::eventhandling::{EventHandlerThread, EventHandlerTrait, EventOrStopThread, Sender};
use crate::threading::{AsyncJoinCallBackTrait, ThreadBuilder};
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
    type TcpSender = RealTcpStream;
    type TcpReceiver = RealTcpStream;

    fn now(&self) -> TimeValue {
        return TimeValue::from_seconds_since_epoch(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64());
    }

    fn new_channel<T: Send>(&self) -> Channel<Self, T> {
        let (sender, receiver) = mpsc::channel::<(SendMetaData, T)>();
        let sender = RealSender::new(self.clone(), sender);
        let receiver = Receiver::new(self.clone(), receiver);
        return Channel::new(sender, receiver);
    }

    fn spawn_event_handler<T: Send, U: EventHandlerTrait<Event=T>>(&self, thread_builder: ThreadBuilder<Self>, channel: Channel<Self, EventOrStopThread<T>>, event_handler: U, join_call_back: impl AsyncJoinCallBackTrait<Self, U::ThreadReturn>) -> std::io::Result<Sender<Self, T>> {
        let (sender, receiver) = channel.take();

        let thread = EventHandlerThread::new(
            receiver,
            event_handler
        );

        thread_builder.spawn_thread(thread, join_call_back)?;

        return Ok(sender);
    }

    //TODO: pass in thread builder
    //TODO: call from thread builder
    fn spawn_tcp_listener<T: TcpConnectionHandlerTrait<TcpSender=Self::TcpSender, TcpReceiver=Self::TcpReceiver>>(&self, socket_addr: SocketAddr, tcp_connection_handler: T, join_call_back: impl AsyncJoinCallBackTrait<Self, T>) -> Result<Sender<Self, ()>, Error> {
        let tcp_listener = TcpListener::bind(socket_addr)?;

        let event_handler = TcpListenerEventHandler::new(tcp_listener, tcp_connection_handler);

        return self.new_thread_builder()
            .name("TODO-NAME-THIS-TCP-LISTENER-THREAD")
            .spawn_event_handler(event_handler, join_call_back);
    }

    fn connect_tcp(&self, socket_addr: SocketAddr) -> Result<(Self::TcpSender, Self::TcpReceiver), Error> {
        let tcp_stream = TcpStream::connect(socket_addr.clone())?;
        let real_tcp_stream = RealTcpStream::new(tcp_stream, socket_addr);
        return Ok((real_tcp_stream.try_clone()?, real_tcp_stream));
    }

    //TODO: pass in thread builder
    //TODO: call from thread builder
    fn spawn_tcp_reader<T: TcpReadHandlerTrait>(&self, tcp_reader: Self::TcpReceiver, tcp_read_handler: T, join_call_back: impl AsyncJoinCallBackTrait<Self, T>) -> Result<Sender<Self, ()>, Error> {

        let event_handler = TcpReaderEventHandler::new(tcp_reader, tcp_read_handler);

        return self.new_thread_builder()
            .name("TODO-NAME-THIS-TCP-READ-THREAD")
            .spawn_event_handler(event_handler, join_call_back);
    }
}