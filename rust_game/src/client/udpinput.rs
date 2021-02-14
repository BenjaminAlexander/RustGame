use std::net::{UdpSocket, Ipv4Addr, SocketAddrV4, SocketAddr};
use crate::gametime::{TimeMessage, TimeReceived, TimeValue, TimeDuration};
use crate::messaging::{InputMessage, StateMessage, ToClientMessageUDP, MAX_UDP_DATAGRAM_SIZE};
use crate::threading::{ConsumerList, Consumer, Sender, Receiver, ChannelThread};
use crate::interface::{State, Input};
use crate::threading::sender::SendError;
use rmp_serde::decode::Error;
use std::io;
use log::{error, info, warn};
use std::time::Duration;

pub struct UdpInput<StateType, InputType>
    where StateType: State<InputType>,
          InputType: Input {

    server_socket_addr: SocketAddr,
    socket: UdpSocket,
    time_message_consumers: ConsumerList<TimeReceived<TimeMessage>>,
    input_message_consumers: ConsumerList<InputMessage<InputType>>,
    state_message_consumers: ConsumerList<StateMessage<StateType>>,

    //metrics
    time_of_last_state_receive: TimeValue,
    time_of_last_input_receive: TimeValue,
}

impl<StateType, InputType> UdpInput<StateType, InputType>
    where StateType: State<InputType>,
          InputType: Input {

    pub fn new(server_socket_addr_v4: SocketAddrV4, socket: &UdpSocket) -> io::Result<Self> {

        let server_socket_addr = SocketAddr::from(server_socket_addr_v4);

        return Ok(Self{
            server_socket_addr,
            socket: socket.try_clone()?,
            time_message_consumers: ConsumerList::new(),
            input_message_consumers: ConsumerList::new(),
            state_message_consumers: ConsumerList::new(),

            //metrics
            time_of_last_state_receive: TimeValue::now(),
            time_of_last_input_receive: TimeValue::now(),
        });
    }
}

impl<StateType, InputType> ChannelThread<()> for UdpInput<StateType, InputType>
    where StateType: State<InputType>,
          InputType: Input {

    fn run(mut self, receiver: Receiver<Self>) {
        info!("Starting");

        let receiver = receiver;

        self.socket.set_read_timeout(Some(Duration::new(1, 0))).unwrap();

        loop {

            let now = TimeValue::now();
            let duration_since_last_state = now.duration_since(self.time_of_last_state_receive);
            if duration_since_last_state > TimeDuration::one_second() {
                warn!("It has been {:?} since last state message was received. Now: {:?}, Last: {:?}",
                      duration_since_last_state, now, self.time_of_last_state_receive);
            }

            let mut buf = [0; MAX_UDP_DATAGRAM_SIZE];

            let recv_result = self.socket.recv_from(&mut buf);
            if recv_result.is_err() {
                warn!("Error on socket recv: {:?}", recv_result);
                continue;
            }

            let (number_of_bytes, source) = recv_result.unwrap();

            if !self.server_socket_addr.eq(&source) {
                warn!("Received from wrong source. Expected: {:?}, Actual: {:?}", self.server_socket_addr, source);
                continue;
            }

            let filled_buf = &mut buf[..number_of_bytes];

            let result: Result<ToClientMessageUDP::<StateType, InputType>, Error> = rmp_serde::from_read_ref(filled_buf);

            match result {
                Ok(message) => {

                    //Why does this crash the client?
                    //info!("{:?}", message);

                    let time_received = TimeValue::now();

                    receiver.try_iter(&mut self);

                    match message {
                        ToClientMessageUDP::TimeMessage(time_message) => {
                            //info!("Time message: {:?}", time_message.get_step());
                            self.time_message_consumers.accept(&TimeReceived::new(time_received, time_message));

                        }
                        ToClientMessageUDP::InputMessage(input_message) => {
                            //TODO: ignore input messages from this player
                            //info!("Input message: {:?}", input_message.get_step());
                            self.time_of_last_input_receive = TimeValue::now();
                            self.input_message_consumers.accept(&input_message);

                        }
                        ToClientMessageUDP::StateMessage(state_message) => {
                            //info!("State message: {:?}", state_message.get_sequence());
                            self.time_of_last_state_receive = TimeValue::now();
                            self.state_message_consumers.accept(&state_message);

                        }
                    }
                }
                Err(error) => {
                    error!("Error: {:?}", error);
                    return;
                }
            }
        }
    }
}
impl<StateType, InputType> Sender<UdpInput<StateType, InputType>>
    where StateType: State<InputType>,
          InputType: Input {

    pub fn add_time_message_consumer<T>(&self, consumer: T) -> Result<(), SendError<UdpInput<StateType, InputType>>>
        where T: Consumer<TimeReceived<TimeMessage>> {

        self.send(|tcp_input|{
            tcp_input.time_message_consumers.add_consumer(consumer);
        })
    }

    pub fn add_input_message_consumer<T>(&self, consumer: T) -> Result<(), SendError<UdpInput<StateType, InputType>>>
        where T: Consumer<InputMessage<InputType>> {

        self.send(|tcp_input|{
            tcp_input.input_message_consumers.add_consumer(consumer);
        })
    }

    pub fn add_state_message_consumer<T>(&self, consumer: T) -> Result<(), SendError<UdpInput<StateType, InputType>>>
        where T: Consumer<StateMessage<StateType>> {

        self.send(|tcp_input|{
            tcp_input.state_message_consumers.add_consumer(consumer);
        })
    }
}