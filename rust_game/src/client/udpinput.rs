use std::net::UdpSocket;
use crate::gametime::{TimeMessage, TimeReceived};
use crate::messaging::{InputMessage, StateMessage, ToClientMessageUDP, MAX_UDP_DATAGRAM_SIZE};
use crate::threading::{ConsumerList, Consumer, Sender, Receiver, ChannelThread};
use crate::interface::{State, Input};
use crate::threading::sender::SendError;
use rmp_serde::decode::Error;
use std::io;
use log::{error, info, warn};

pub struct UdpInput<StateType, InputType>
    where StateType: State<InputType>,
          InputType: Input {

    port: u16,
    socket: UdpSocket,
    time_message_consumers: ConsumerList<TimeReceived<TimeMessage>>,
    input_message_consumers: ConsumerList<InputMessage<InputType>>,
    state_message_consumers: ConsumerList<StateMessage<StateType>>,
}

impl<StateType, InputType> UdpInput<StateType, InputType>
    where StateType: State<InputType>,
          InputType: Input {

    pub fn new(port: u16, socket: &UdpSocket) -> io::Result<Self> {
        return Ok(Self{
            port,
            socket: socket.try_clone()?,
            time_message_consumers: ConsumerList::new(),
            input_message_consumers: ConsumerList::new(),
            state_message_consumers: ConsumerList::new(),
        });
    }
}

impl<StateType, InputType> ChannelThread<()> for UdpInput<StateType, InputType>
    where StateType: State<InputType>,
          InputType: Input {

    fn run(mut self, receiver: Receiver<Self>) {
        info!("Starting");

        let mut buf = [0; MAX_UDP_DATAGRAM_SIZE];
        let (number_of_bytes, source) = self.socket.recv_from(&mut buf).unwrap();
        let filled_buf = &mut buf[..number_of_bytes];

        let result: Result<ToClientMessageUDP::<StateType, InputType>, Error> = rmp_serde::from_read_ref(filled_buf);


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