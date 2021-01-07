use log::{error, info};
use std::net::TcpStream;
use crate::gametime::{TimeMessage, TimeReceived, TimeValue};
use crate::threading::{ConsumerList, ChannelThread, Receiver, Sender, Consumer};
use crate::messaging::{ToClientMessage, InputMessage, StateMessage};
use rmp_serde::decode::Error;
use crate::threading::sender::SendError;
use std::io;
use crate::interface::{Input, State};
use std::marker::PhantomData;

pub struct TcpInput <StateType, InputType>
    where InputType: Input,
          StateType: State {

    tcp_stream: TcpStream,
    time_message_consumers: ConsumerList<TimeReceived<TimeMessage>>,
    input_message_consumers: ConsumerList<InputMessage<InputType>>,
    state_message_consumers: ConsumerList<StateMessage<StateType>>,
    phantom: PhantomData<InputType>,
    state_phantom: PhantomData<StateType>,
}

impl<StateType, InputType> TcpInput<StateType, InputType>
    where InputType: Input,
          StateType: State {

    pub fn new(tcp_stream: &TcpStream) -> io::Result<Self> {
        Ok(Self {
            tcp_stream: tcp_stream.try_clone()?,
            time_message_consumers: ConsumerList::new(),
            input_message_consumers: ConsumerList::new(),
            state_message_consumers: ConsumerList::new(),
            phantom: PhantomData,
            state_phantom: PhantomData
        })
    }
}

impl<StateType, InputType> ChannelThread<()> for TcpInput<StateType, InputType>
    where InputType: Input,
          StateType: State {

    fn run(mut self, receiver: Receiver<Self>) {
        info!("Starting");

        let receiver = receiver;

        loop {
            let result: Result<ToClientMessage::<StateType, InputType>, Error> = rmp_serde::from_read(&self.tcp_stream);

            match result {
                Ok(message) => {
                    let time_received = TimeValue::now();

                    receiver.try_iter(&mut self);

                    match message {
                        ToClientMessage::TimeMessage(time_message) => {
                            self.time_message_consumers.accept(&TimeReceived::new(time_received, time_message));

                        }
                        ToClientMessage::InputMessage(input_message) => {
                            self.input_message_consumers.accept(&input_message);

                        }
                        ToClientMessage::StateMessage(state_message) => {
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

impl<StateType, InputType> Sender<TcpInput<StateType, InputType>>
    where InputType: Input,
          StateType: State {

    pub fn add_time_message_consumer<T>(&self, consumer: T) -> Result<(), SendError<TcpInput<StateType, InputType>>>
        where T: Consumer<TimeReceived<TimeMessage>> {

        self.send(|tcp_input|{
            tcp_input.time_message_consumers.add_consumer(consumer);
        })
    }

    pub fn add_input_message_consumer<T>(&self, consumer: T) -> Result<(), SendError<TcpInput<StateType, InputType>>>
        where T: Consumer<InputMessage<InputType>> {

        self.send(|tcp_input|{
            tcp_input.input_message_consumers.add_consumer(consumer);
        })
    }

    pub fn add_state_message_consumer<T>(&self, consumer: T) -> Result<(), SendError<TcpInput<StateType, InputType>>>
        where T: Consumer<StateMessage<StateType>> {

        self.send(|tcp_input|{
            tcp_input.state_message_consumers.add_consumer(consumer);
        })
    }
}