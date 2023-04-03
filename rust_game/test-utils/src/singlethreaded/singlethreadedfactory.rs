use std::io::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::mpsc;
use commons::factory::FactoryTrait;
use commons::net::{TcpConnectionHandlerTrait, TcpReadHandlerTrait};
use commons::threading::AsyncJoinCallBackTrait;
use commons::threading::channel::{Channel, ChannelThreadBuilder, RealSender, Receiver, SendMetaData};
use commons::threading::eventhandling::{EventHandlerTrait, EventOrStopThread, Sender};
use commons::time::TimeValue;
use crate::net::{ChannelTcpReader, ChannelTcpWriter, HostSimulator, NetworkSimulator};
use crate::singlethreaded::eventhandling::EventHandlerHolder;
use crate::singlethreaded::{SingleThreadedSender, TimeQueue};
use crate::time::SimulatedTimeSource;

#[derive(Clone)]
pub struct SingleThreadedFactory {
    //TODO: don't let this SimulatedTimeSource escape SingleThreaded package
    simulated_time_source: SimulatedTimeSource,
    //TODO: don't let this TimeQueue escape SingleThreaded package
    time_queue: TimeQueue,
    host_simulator: HostSimulator
}

impl SingleThreadedFactory {

    pub fn new() -> Self {

        let simulated_time_source = SimulatedTimeSource::new();
        let time_queue = TimeQueue::new(simulated_time_source.clone());

        return Self {
            simulated_time_source,
            host_simulator: NetworkSimulator::new(time_queue.clone()).new_host(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            time_queue,
        }
    }

    pub fn get_simulated_time_source(&self) -> &SimulatedTimeSource {
        return &self.simulated_time_source;
    }

    pub fn get_time_queue(&self) -> &TimeQueue {
        return &self.time_queue;
    }

    pub fn get_host_simulator(&self) -> &HostSimulator {
        return &self.host_simulator;
    }

    pub fn clone_for_new_host(&self, ip_adder: IpAddr) -> Self {
        let mut clone = self.clone();
        clone.host_simulator = clone.host_simulator.get_network_simulator().new_host(ip_adder);
        return clone;
    }

}

impl FactoryTrait for SingleThreadedFactory {
    type Sender<T: Send> = SingleThreadedSender<T>;
    type TcpWriter = ChannelTcpWriter;
    type TcpReader = ChannelTcpReader;

    fn now(&self) -> TimeValue {
        return self.simulated_time_source.now();
    }

    fn new_channel<T: Send>(&self) -> Channel<Self, T> {
        let (sender, receiver) = mpsc::channel::<(SendMetaData, T)>();
        let sender = RealSender::new(self.clone(), sender);
        let sender = SingleThreadedSender::new(sender);
        let receiver = Receiver::new(self.clone(), receiver);
        return Channel::new(sender, receiver);
    }

    fn spawn_event_handler< U: EventHandlerTrait>(&self, thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<U::Event>>, event_handler: U, join_call_back: impl AsyncJoinCallBackTrait<Self, U::ThreadReturn>) -> Result<Sender<Self, U::Event>, Error> {

        return Ok(EventHandlerHolder::spawn_event_handler(
            self.clone(),
            thread_builder,
            event_handler,
            join_call_back));
    }

    fn spawn_tcp_listener<T: TcpConnectionHandlerTrait<Factory=Self>>(
        &self,
        thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<()>>,
        socket_addr: SocketAddr,
        tcp_connection_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Self, T>) -> Result<Sender<Self, ()>, Error> {

        return self.host_simulator.get_network_simulator().spawn_tcp_listener(
            self,
            socket_addr,
            thread_builder,
            tcp_connection_handler,
            join_call_back);

    }

    fn connect_tcp(&self, socket_addr: SocketAddr) -> Result<(Self::TcpWriter, Self::TcpReader), Error> {
        return self.host_simulator.connect_tcp(&self, socket_addr);
    }

    fn spawn_tcp_reader<T: TcpReadHandlerTrait>(
        &self,
        thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<()>>,
        tcp_reader: Self::TcpReader,
        read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<Self, T>) -> Result<Sender<Self, ()>, Error> {

        todo!()
    }
}