use crate::factory::FactoryTrait;
use crate::net::{
    TcpConnectionHandlerTrait,
    TcpReadHandlerTrait,
    TcpStream,
    UdpReadHandlerTrait,
};
use crate::single_threaded_simulator::eventhandling::EventHandlerHolder;
use crate::single_threaded_simulator::net::{
    HostSimulator,
    NetworkSimulator,
    TcpReaderEventHandler,
    UdpSocketSimulator,
};
use crate::single_threaded_simulator::{
    ReceiveOrDisconnected,
    SingleThreadedReceiver,
    SingleThreadedSender,
    TimeQueue,
};
use crate::threading::channel::{
    Channel,
    ChannelThreadBuilder,
};
use crate::threading::eventhandling::{
    EventHandlerSender,
    EventHandlerTrait,
    EventOrStopThread,
    EventSenderTrait,
};
use crate::threading::AsyncJoinCallBackTrait;
use crate::time::{
    SimulatedTimeSource,
    TimeSource,
};
use std::io::Error;
use std::net::{
    IpAddr,
    Ipv4Addr,
    SocketAddr,
};

#[derive(Clone)]
pub struct SingleThreadedFactory {
    time_source: TimeSource,
    simulated_time_source: SimulatedTimeSource,
    //TODO: don't let this TimeQueue escape SingleThreaded package
    time_queue: TimeQueue,
    host_simulator: HostSimulator,
}

impl SingleThreadedFactory {
    pub fn new() -> Self {
        let (time_source, simulated_time_source) = TimeSource::new_simulated_time_source();
        let time_queue = TimeQueue::new(simulated_time_source.clone());

        return Self {
            time_source,
            simulated_time_source,
            host_simulator: NetworkSimulator::new()
                .new_host(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            time_queue,
        };
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
        clone.host_simulator = clone
            .host_simulator
            .get_network_simulator()
            .new_host(ip_adder);
        return clone;
    }
}

impl FactoryTrait for SingleThreadedFactory {
    type Sender<T: Send> = SingleThreadedSender<T>;
    type Receiver<T: Send> = SingleThreadedReceiver<T>;

    type TcpReader = SingleThreadedReceiver<Vec<u8>>;

    type UdpSocket = UdpSocketSimulator;

    fn get_time_source(&self) -> &TimeSource {
        return &self.time_source;
    }

    fn new_channel<T: Send>(&self) -> Channel<Self, T> {
        let (sender, receiver) = SingleThreadedReceiver::new(self.clone());
        return Channel::new(sender, receiver);
    }

    fn spawn_event_handler<U: EventHandlerTrait>(
        &self,
        thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<U::Event>>,
        event_handler: U,
        join_call_back: impl AsyncJoinCallBackTrait<U::ThreadReturn>,
    ) -> Result<EventHandlerSender<Self, U::Event>, Error> {
        return Ok(EventHandlerHolder::spawn_event_handler(
            self.clone(),
            thread_builder,
            event_handler,
            join_call_back,
        ));
    }

    fn spawn_tcp_listener<T: TcpConnectionHandlerTrait<Self>>(
        &self,
        thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<()>>,
        socket_addr: SocketAddr,
        tcp_connection_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T>,
    ) -> Result<EventHandlerSender<Self, ()>, Error> {
        return self
            .host_simulator
            .get_network_simulator()
            .spawn_tcp_listener(
                self.clone(),
                socket_addr,
                thread_builder,
                tcp_connection_handler,
                join_call_back,
            );
    }

    fn connect_tcp(&self, socket_addr: SocketAddr) -> Result<(TcpStream, Self::TcpReader), Error> {
        return self.host_simulator.connect_tcp(&self, socket_addr);
    }

    fn spawn_tcp_reader<T: TcpReadHandlerTrait>(
        &self,
        thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<()>>,
        tcp_reader: Self::TcpReader,
        read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T>,
    ) -> Result<EventHandlerSender<Self, ()>, Error> {
        let (thread_builder, channel) = thread_builder.take();

        let tcp_reader_event_handler = TcpReaderEventHandler::new(read_handler);

        let sender = thread_builder
            .spawn_event_handler(self.clone(), tcp_reader_event_handler, join_call_back)
            .unwrap();

        let sender_clone = sender.clone();
        tcp_reader.to_consumer(move |receive_or_disconnect| {
            match receive_or_disconnect {
                ReceiveOrDisconnected::Receive(_, buf) => {
                    return match sender_clone.send_event(buf) {
                        Ok(_) => Ok(()),
                        Err(buf) => Err(buf),
                    };
                }
                ReceiveOrDisconnected::Disconnected => {
                    let _ = sender_clone.send_stop_thread();
                    return Ok(());
                }
            };
        });

        let (sender_to_return, receiver) = channel.take();

        receiver.to_consumer(move |receive_or_disconnect| {
            let result = match receive_or_disconnect {
                ReceiveOrDisconnected::Receive(_, EventOrStopThread::Event(())) => Ok(()),
                ReceiveOrDisconnected::Receive(_, EventOrStopThread::StopThread) => {
                    sender.send_stop_thread()
                }
                ReceiveOrDisconnected::Disconnected => sender.send_stop_thread(),
            };

            return match result {
                Ok(()) => Ok(()),
                Err(_) => Err(EventOrStopThread::StopThread),
            };
        });

        return Ok(sender_to_return);
    }

    fn bind_udp_socket(&self, socket_addr: SocketAddr) -> Result<Self::UdpSocket, Error> {
        return self.host_simulator.bind_udp_socket(socket_addr);
    }

    fn spawn_udp_reader<T: UdpReadHandlerTrait>(
        &self,
        thread_builder: ChannelThreadBuilder<Self, EventOrStopThread<()>>,
        udp_socket: Self::UdpSocket,
        udp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T>,
    ) -> Result<EventHandlerSender<Self, ()>, Error> {
        return self
            .host_simulator
            .get_network_simulator()
            .spawn_udp_reader(self.clone(), thread_builder, udp_socket, udp_read_handler, join_call_back);
    }
}
