use std::net::TcpStream;
use log::{info, error};
use crate::threading::{Sender, ReceiverBundle, Consumer, ConsumerList, ChannelThread, Receiver};
use crate::threading;
use crate::simplegame::Vector2;
use crate::messaging::{Message, ToServerMessage};
use rmp_serde::decode::{Error, ReadReader};
use rmp_serde::Deserializer as RmpDeserializer;
use serde::de::{Deserializer as SerdeDeserializer, DeserializeOwned};
use std::io::{Read, BufReader, BufRead};
use std::marker::PhantomData;
use std::fmt::Debug;
use std::time::SystemTime;
use crate::server::timedinputmessage::TimedInputMessage;
use crate::interface::Input;

pub struct TcpInput<InputType>
    where InputType: Input {

    tcp_stream: TcpStream,
    input_consumers: ConsumerList<TimedInputMessage<InputType>>
}

impl<InputType> TcpInput<InputType>
    where InputType: Input {

    pub fn new(tcp_stream: TcpStream) -> Self {
        Self { tcp_stream, input_consumers: ConsumerList::new() }
    }
}

impl<InputType> ChannelThread<()> for TcpInput<InputType>
    where InputType: Input {

    fn run(mut self, receiver: Receiver<Self>) {
        info!("Starting");

        let mut receiver = receiver;

        loop {
            let result: Result<ToServerMessage<InputType>, Error> = rmp_serde::from_read(&self.tcp_stream);

            match result {
                Ok(message) => {
                    let time_received = SystemTime::now();

                    receiver.try_iter(&mut self);

                    match message {
                        ToServerMessage::Input(inputMessage) => {

                            let timed_message = TimedInputMessage::new(inputMessage, time_received);

                            self.input_consumers.accept(&timed_message);
                        }
                    }
                }
                Err(error) => {
                    error!("Ending due to: {:?}", error);
                    return;
                }
            }

            receiver.try_iter(&mut self);


        }

        info!("Ending");
    }
}

impl<InputType> Sender<TcpInput<InputType>>
    where InputType: Input
{
    pub fn add_input_consumer<T>(&self, consumer: T)
        where T: Consumer<TimedInputMessage<InputType>>
    {
        self.send(|tcp_input|{
            tcp_input.input_consumers.add_consumer(consumer);
        });
    }
}

pub struct TestConsumer;

impl<InputType> Consumer<TimedInputMessage<InputType>> for TestConsumer
    where InputType: Input {
    fn accept(&self, t: TimedInputMessage<InputType>) {
        info!("Consume {:?}", t);
    }
}