use std::io::Error;
use std::net::ToSocketAddrs;
use std::sync::mpsc;
use commons::factory::FactoryTrait;
use commons::net::{RealTcpListener, TcpConnectionHandler, TcpListenerTrait};
use commons::threading::{AsyncJoinCallBackTrait, ThreadBuilder};
use commons::threading::channel::{Channel, RealSender, Receiver, SendMetaData};
use commons::threading::eventhandling::{EventHandlerTrait, EventOrStopThread, Sender};
use commons::time::TimeValue;
use crate::singlethreaded::eventhandling::EventHandlerHolder;
use crate::singlethreaded::{SingleThreadedSender, TimeQueue};
use crate::time::SimulatedTimeSource;

#[derive(Clone)]
pub struct SingleThreadedFactory {
    //TODO: don't let this SimulatedTimeSource escape SingleThreaded package
    simulated_time_source: SimulatedTimeSource,
    //TODO: don't let this TimeQueue escape SingleThreaded package
    time_queue: TimeQueue
}

impl SingleThreadedFactory {

    pub fn new() -> Self {

        let simulated_time_source = SimulatedTimeSource::new();
        let time_queue = TimeQueue::new(simulated_time_source.clone());

        return Self {
            simulated_time_source,
            time_queue
        }
    }

    pub fn get_simulated_time_source(&self) -> &SimulatedTimeSource {
        return &self.simulated_time_source;
    }

    pub fn get_time_queue(&self) -> &TimeQueue {
        return &self.time_queue;
    }
}

impl FactoryTrait for SingleThreadedFactory {
    type Sender<T: Send> = SingleThreadedSender<T>;

    //TODO: make a fake listener
    type TcpListener = RealTcpListener;

    fn now(&self) -> TimeValue {
        return self.simulated_time_source.now();
    }

    fn new_channel<T: Send>(&self) -> Channel<Self, T> {
        let (sender, receiver) = mpsc::channel::<(SendMetaData, T)>();
        let sender = RealSender::new(self.clone(), sender);
        let sender = SingleThreadedSender::new(sender);
        let receiver = Receiver::new(receiver);
        return Channel::new(sender, receiver);
    }

    fn spawn_event_handler<T: Send, U: EventHandlerTrait<Event=T>>(&self, thread_builder: ThreadBuilder<Self>, channel: Channel<Self, EventOrStopThread<T>>, event_handler: U, join_call_back: impl AsyncJoinCallBackTrait<Self, U::ThreadReturn>) -> Result<Sender<Self, T>, Error> {
        let (sender, receiver) = channel.take();

        let event_handler_holder = EventHandlerHolder::new(
            self.clone(),
            thread_builder,
            receiver,
            event_handler,
            join_call_back);

        sender.set_on_send(move ||{
            event_handler_holder.on_send();
        });

        return Ok(sender);
    }

    fn new_tcp_listener(&self, socket_addr: impl ToSocketAddrs) -> Result<Self::TcpListener, Error> {
        return RealTcpListener::bind(socket_addr);
    }

    fn spawn_tcp_listener<T: TcpConnectionHandler<TcpStream=<Self::TcpListener as TcpListenerTrait>::TcpStream>>(&self, tcp_listener: Self::TcpListener, tcp_connection_handler: T, join_call_back: impl AsyncJoinCallBackTrait<Self, T>) -> Result<Sender<Self, ()>, Error> {
        todo!()
    }
}