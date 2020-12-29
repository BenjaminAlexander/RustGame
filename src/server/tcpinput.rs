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

pub struct TcpInput<InputType>
    where InputType: Clone + Debug + DeserializeOwned + Send + 'static {

    tcpStream: TcpStream,
    inputConsumers: ConsumerList<TimedInputMessage<InputType>>
}

impl<InputType> TcpInput<InputType>
    where InputType: Clone + Debug + DeserializeOwned + Send + 'static {

    pub fn new(tcpStream: TcpStream) -> Self {
        Self { tcpStream, inputConsumers: ConsumerList::new() }
    }
}

impl<InputType> ChannelThread<()> for TcpInput<InputType>
    where InputType: Clone + Debug + DeserializeOwned + Send + 'static {

    fn run(mut self, receiver: Receiver<Self>) {
        info!("Starting");

        let mut receiver = receiver;

        loop {
            let result: Result<ToServerMessage<InputType>, Error> = rmp_serde::from_read(&self.tcpStream);

            match result {
                Ok(message) => {
                    let timeReceived = SystemTime::now();

                    receiver.try_iter(&mut self);

                    match message {
                        ToServerMessage::Input(inputMessage) => {

                            let timedMessage = TimedInputMessage::new(inputMessage, timeReceived);

                            self.inputConsumers.accept(&timedMessage);
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
    where InputType: Clone + Debug + DeserializeOwned + Send + 'static
{
    pub fn add_input_consumer<T>(&self, consumer: T)
        where T: Consumer<TimedInputMessage<InputType>>
    {
        self.send(|tcpInput|{
            tcpInput.inputConsumers.add_consumer(consumer);
        });
    }
}

pub struct TestConsumer;

impl<InputType> Consumer<TimedInputMessage<InputType>> for TestConsumer
    where InputType: Clone + Debug {
    fn accept(&self, t: TimedInputMessage<InputType>) {
        info!("Consume {:?}", t);
    }
}