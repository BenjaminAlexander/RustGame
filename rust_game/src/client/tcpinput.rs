use log::{error, info, warn};
use std::net::TcpStream;
use crate::gametime::{TimeMessage, TimeReceived, TimeValue, TimeDuration};
use crate::threading::{ConsumerList, ChannelThread, Receiver, Sender, Consumer};
use crate::messaging::{ToClientMessageTCP, InputMessage, StateMessage, InitialInformation};
use rmp_serde::decode::Error;
use crate::threading::sender::SendError;
use std::io;
use crate::interface::{Input, State, InputEvent, Game};
use std::marker::PhantomData;

pub struct TcpInput <GameType: Game> {
    player_index: Option<usize>,
    tcp_stream: TcpStream,
    initial_information_message_consumers: ConsumerList<InitialInformation<GameType>>
}

impl<GameType: Game> TcpInput<GameType> {

    pub fn new(tcp_stream: &TcpStream) -> io::Result<Self> {
        Ok(Self {
            player_index: None,
            tcp_stream: tcp_stream.try_clone()?,
            initial_information_message_consumers: ConsumerList::new(),
        })
    }
}

impl<GameType: Game> ChannelThread<()> for TcpInput<GameType> {

    fn run(mut self, receiver: Receiver<Self>) {
        info!("Starting");

        let receiver = receiver;

        loop {
            let result: Result<ToClientMessageTCP::<GameType>, Error> = rmp_serde::from_read(&self.tcp_stream);

            match result {
                Ok(message) => {

                    //Why does this crash the client?
                    //info!("{:?}", message);

                    let time_received = TimeValue::now();

                    receiver.try_iter(&mut self);

                    match message {
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
        }
    }
}

impl<GameType: Game> Sender<TcpInput<GameType>> {

    pub fn add_initial_information_message_consumer<T>(&self, consumer: T) -> Result<(), SendError<TcpInput<GameType>>>
        where T: Consumer<InitialInformation<GameType>> {

        self.send(|tcp_input|{
            tcp_input.initial_information_message_consumers.add_consumer(consumer);
        })
    }
}