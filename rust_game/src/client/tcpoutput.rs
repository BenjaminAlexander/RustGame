use std::net::TcpStream;
use crate::threading::{ChannelDrivenThread, Consumer, Sender};
use crate::interface::{Input, InputEvent};
use crate::messaging::{InputMessage, ToServerMessage};
use std::io;
use std::marker::PhantomData;

//TODO: Send response to time messages to calculate ping

pub struct TcpOutput<InputType, InputEventType>
    where InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    tcp_stream: TcpStream,
    input_queue: Vec<InputMessage<InputType>>,
    event_phantom: PhantomData<InputEventType>
}

impl<InputType, InputEventType> TcpOutput<InputType, InputEventType>
    where InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    pub fn new(tcp_stream: &TcpStream) -> io::Result<Self> {
        Ok(Self{tcp_stream: tcp_stream.try_clone()?, input_queue: Vec::new(), event_phantom: PhantomData})
    }
}

impl<InputType, InputEventType> ChannelDrivenThread<()> for TcpOutput<InputType, InputEventType>
    where InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl<InputType, InputEventType> Consumer<InputMessage<InputType>> for Sender<TcpOutput<InputType, InputEventType>>
    where InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    fn accept(&self, input_message: InputMessage<InputType>) {
        self.send(move |tcp_output|{

            let message = ToServerMessage::Input(input_message);
            rmp_serde::encode::write(&mut tcp_output.tcp_stream, &message).unwrap();

        }).unwrap();
    }
}