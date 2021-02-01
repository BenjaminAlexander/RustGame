use std::net::TcpStream;

use log::{error, info};
use rmp_serde::decode::Error;

use crate::interface::{Input, InputEvent};
use crate::messaging::{ToServerMessage, InputMessage};
use crate::threading::{ChannelThread, Consumer, ConsumerList, Receiver, Sender};
use crate::gametime::{TimeReceived, TimeValue};
use std::io;
use crate::threading::sender::SendError;
use serde::export::PhantomData;

pub struct TcpInput<InputType, InputEventType>
    where InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    tcp_stream: TcpStream,
    input_consumers: ConsumerList<InputMessage<InputType>>,
    phantom: PhantomData<InputEventType>
}

impl<InputType, InputEventType> TcpInput<InputType, InputEventType>
    where InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    pub fn new(tcp_stream: &TcpStream) -> io::Result<Self> {
        Ok(Self {tcp_stream: tcp_stream.try_clone()?, input_consumers: ConsumerList::new(), phantom: PhantomData})
    }
}

impl<InputType, InputEventType> ChannelThread<()> for TcpInput<InputType, InputEventType>
    where InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    fn run(mut self, receiver: Receiver<Self>) {
        info!("Starting");

        let receiver = receiver;

        loop {
            let result: Result<ToServerMessage<InputType>, Error> = rmp_serde::from_read(&self.tcp_stream);

            //TODO: check player ID on message

            match result {
                Ok(message) => {
                    let time_received = TimeValue::now();

                    receiver.try_iter(&mut self);

                    match message {
                        ToServerMessage::Input(input_message) => {
                            error!("Recieved Message!!!!!!!!!!!!!");
                            self.input_consumers.accept(&input_message);
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
    }
}

impl<InputType, InputEventType> Sender<TcpInput<InputType, InputEventType>>
    where InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    pub fn add_input_consumer<T>(&self, consumer: T) -> Result<(), SendError<TcpInput<InputType, InputEventType>>>
        where T: Consumer<InputMessage<InputType>> {

        self.send(|tcp_input|{
            tcp_input.input_consumers.add_consumer(consumer);
        })
    }
}