use log::{error, info};
use std::net::TcpStream;
use crate::gametime::{TimeMessage, TimeReceived, TimeValue};
use crate::threading::{ConsumerList, ChannelThread, Receiver, Sender, Consumer};
use crate::messaging::ToClientMessage;
use rmp_serde::decode::Error;
use crate::threading::sender::SendError;
use std::io;
use crate::interface::Input;
use std::marker::PhantomData;

pub struct TcpInput <InputType>
    where InputType: Input {

    tcp_stream: TcpStream,
    time_message_consumers: ConsumerList<TimeReceived<TimeMessage>>,
    phantom: PhantomData<InputType>
}

impl<InputType> TcpInput<InputType>
    where InputType: Input {

    pub fn new(tcp_stream: &TcpStream) -> io::Result<Self> {
        Ok(Self {
            tcp_stream: tcp_stream.try_clone()?,
            time_message_consumers: ConsumerList::new(),
            phantom: PhantomData
        })
    }
}

impl<InputType> ChannelThread<()> for TcpInput<InputType>
    where InputType: Input {

    fn run(mut self, receiver: Receiver<Self>) {
        info!("Starting");

        let receiver = receiver;

        loop {
            let result: Result<ToClientMessage::<InputType>, Error> = rmp_serde::from_read(&self.tcp_stream);

            match result {
                Ok(message) => {
                    let time_received = TimeValue::now();

                    receiver.try_iter(&mut self);

                    match message {
                        ToClientMessage::TimeMessage(time_message) => {

                            let timed_message = TimeReceived::new(time_received, time_message);
                            self.time_message_consumers.accept(&timed_message);
                        }
                        ToClientMessage::InputMessage(input_message) => {
                            //TODO: handle input message
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

impl<InputType> Sender<TcpInput<InputType>>
    where InputType: Input {

    pub fn add_time_message_consumer<T>(&self, consumer: T) -> Result<(), SendError<TcpInput<InputType>>>
        where T: Consumer<TimeReceived<TimeMessage>> {

        self.send(|tcp_input|{
            tcp_input.time_message_consumers.add_consumer(consumer);
        })
    }
}