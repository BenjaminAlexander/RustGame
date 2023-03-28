use std::io::Error;
use std::net::ToSocketAddrs;
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::factory::FactoryTrait;
use crate::net::RealTcpListener;
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
    type TcpListener<ReadType: Serialize + DeserializeOwned + Send, WriteType: Serialize + DeserializeOwned + Send> = RealTcpListener<ReadType, WriteType>;

    fn now(&self) -> TimeValue {
        return TimeValue::from_seconds_since_epoch(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64());
    }

    fn new_channel<T: Send>(&self) -> Channel<Self, T> {
        let (sender, receiver) = mpsc::channel::<(SendMetaData, T)>();
        let sender = RealSender::new(self.clone(), sender);
        let receiver = Receiver::new(receiver);
        return Channel::new(sender, receiver);
    }

    fn spawn_event_handler<T: Send, U: EventHandlerTrait<Event=T>>(&self, thread_builder: ThreadBuilder<Self>, channel: Channel<Self, EventOrStopThread<T>>, event_handler: U, join_call_back: impl AsyncJoinCallBackTrait<Self, U::ThreadReturn>) -> std::io::Result<Sender<Self, T>> {
        let (sender, receiver) = channel.take();

        let thread = EventHandlerThread::new(
            thread_builder.get_factory().clone(),
            receiver,
            event_handler
        );

        thread_builder.spawn_thread(thread, join_call_back)?;

        return Ok(sender);
    }

    fn new_tcp_listener<ReadType: Serialize + DeserializeOwned + Send, WriteType: Serialize + DeserializeOwned + Send>(&self, socket_addr: impl ToSocketAddrs) -> Result<Self::TcpListener<ReadType, WriteType>, Error> {
        return RealTcpListener::bind(socket_addr);
    }
}