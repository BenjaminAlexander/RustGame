use log::{error, info, warn};
use std::net::TcpStream;
use crate::gametime::{TimeMessage, TimeReceived, TimeValue, TimeDuration};
use crate::threading::{ConsumerList, ChannelThread, Receiver, Sender, Consumer};
use crate::messaging::{ToClientMessageTCP, InputMessage, StateMessage, InitialInformation};
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
    initial_information_message_consumers: ConsumerList<InitialInformation<StateType>>,

    //metrics
    time_of_last_state_receive: TimeValue,
    time_of_last_input_receive: TimeValue,
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
            initial_information_message_consumers: ConsumerList::new(),

            //metrics
            time_of_last_state_receive: TimeValue::now(),
            time_of_last_input_receive: TimeValue::now(),
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
            let result: Result<ToClientMessageTCP::<StateType, InputType>, Error> = rmp_serde::from_read(&self.tcp_stream);

            match result {
                Ok(message) => {

                    //Why does this crash the client?
                    //info!("{:?}", message);

                    let time_received = TimeValue::now();

                    receiver.try_iter(&mut self);

                    match message {
                        ToClientMessageTCP::TimeMessage(time_message) => {
                            //info!("Time message: {:?}", time_message.get_step());
                            self.time_message_consumers.accept(&TimeReceived::new(time_received, time_message));

                        }
                        ToClientMessageTCP::InputMessage(input_message) => {
                            //TODO: ignore input messages from this player
                            //info!("Input message: {:?}", input_message.get_step());
                            self.time_of_last_input_receive = TimeValue::now();
                            self.input_message_consumers.accept(&input_message);

                        }
                        ToClientMessageTCP::StateMessage(state_message) => {
                            //info!("State message: {:?}", state_message.get_sequence());
                            self.time_of_last_state_receive = TimeValue::now();
                            self.state_message_consumers.accept(&state_message);

                        }
                        ToClientMessageTCP::InitialInformation(initial_information_message) => {
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

            let now = TimeValue::now();
            let duration_since_last_state = now.duration_since(self.time_of_last_state_receive);
            if duration_since_last_state > TimeDuration::one_second() {
                warn!("It has been {:?} since last state message was received. Now: {:?}, Last: {:?}",
                      duration_since_last_state, now, self.time_of_last_state_receive);
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