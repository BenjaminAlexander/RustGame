use std::net::TcpStream;
use std::time::SystemTime;

use log::{error, info};
use rmp_serde::decode::Error;

use crate::interface::Input;
use crate::messaging::ToServerMessage;
use crate::server::timedinputmessage::TimedInputMessage;
use crate::threading::{ChannelThread, Consumer, ConsumerList, Receiver, Sender};
use crate::threading::sender::SendError;

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

        let receiver = receiver;

        loop {
            let result: Result<ToServerMessage<InputType>, Error> = rmp_serde::from_read(&self.tcp_stream);

            match result {
                Ok(message) => {
                    let time_received = SystemTime::now();

                    receiver.try_iter(&mut self);

                    match message {
                        ToServerMessage::Input(input_message) => {

                            let timed_message = TimedInputMessage::new(input_message, time_received);

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
    }
}

impl<InputType> Sender<TcpInput<InputType>>
    where InputType: Input
{
    pub fn add_input_consumer<T>(&self, consumer: T) -> Result<(), SendError<TcpInput<InputType>>>
        where T: Consumer<TimedInputMessage<InputType>> {
        self.send(|tcp_input|{
            tcp_input.input_consumers.add_consumer(consumer);
        })
    }
}

pub struct TestConsumer;

impl<InputType> Consumer<TimedInputMessage<InputType>> for TestConsumer
    where InputType: Input {
    fn accept(&self, t: TimedInputMessage<InputType>) {
        info!("Consume {:?}", t);
    }
}