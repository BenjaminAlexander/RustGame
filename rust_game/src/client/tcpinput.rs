use log::{error, info};
use std::net::TcpStream;
use crate::gametime::{TimeMessage, TimeReceived, TimeValue};
use crate::threading::{ConsumerList, ChannelThread, Receiver, Sender, Consumer};
use crate::messaging::{ToClientMessage, InputMessage, StateMessage, InitialInformation};
use rmp_serde::decode::Error;
use crate::threading::sender::SendError;
use std::io;
use crate::interface::{Input, State, InputEvent};
use std::marker::PhantomData;

pub struct TcpInput <StateType, InputType>
    where StateType: State<InputType>,
          InputType: Input {

    player_index: Option<usize>,
    tcp_stream: TcpStream,
    time_message_consumers: ConsumerList<TimeReceived<TimeMessage>>,
    input_message_consumers: ConsumerList<InputMessage<InputType>>,
    state_message_consumers: ConsumerList<StateMessage<StateType>>,
    initial_information_message_consumers: ConsumerList<InitialInformation<StateType>>
}

impl<StateType, InputType> TcpInput<StateType, InputType>
    where StateType: State<InputType>,
          InputType: Input {

    pub fn new(tcp_stream: &TcpStream) -> io::Result<Self> {
        Ok(Self {
            player_index: None,
            tcp_stream: tcp_stream.try_clone()?,
            time_message_consumers: ConsumerList::new(),
            input_message_consumers: ConsumerList::new(),
            state_message_consumers: ConsumerList::new(),
            initial_information_message_consumers: ConsumerList::new()
        })
    }
}

impl<StateType, InputType> ChannelThread<()> for TcpInput<StateType, InputType>
    where StateType: State<InputType>,
          InputType: Input {

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
                            //TODO: ignore input messages from this player
                            self.input_message_consumers.accept(&input_message);

                        }
                        ToClientMessage::StateMessage(state_message) => {
                            self.state_message_consumers.accept(&state_message);

                        }
                        ToClientMessage::InitialInformation(initial_information_message) => {
                            self.player_index = Some(initial_information_message.get_player_index());
                            self.initial_information_message_consumers.accept(&initial_information_message);
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
    where StateType: State<InputType>,
          InputType: Input {

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

    pub fn add_initial_information_message_consumer<T>(&self, consumer: T) -> Result<(), SendError<TcpInput<StateType, InputType>>>
        where T: Consumer<InitialInformation<StateType>> {

        self.send(|tcp_input|{
            tcp_input.initial_information_message_consumers.add_consumer(consumer);
        })
    }
}