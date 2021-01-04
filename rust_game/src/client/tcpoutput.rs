use std::net::TcpStream;
use crate::threading::{ChannelDrivenThread, Consumer, Sender};
use crate::interface::Input;
use crate::messaging::{InputMessage, ToServerMessage};
use std::io;
use std::marker::PhantomData;

pub struct TcpOutput<InputType>
    where InputType: Input {
    tcp_stream: TcpStream,
    phantom: PhantomData<InputType>
}

impl<InputType> TcpOutput<InputType>
    where InputType: Input {

    pub fn new(tcp_stream: &TcpStream) -> io::Result<Self> {
        Ok(Self{tcp_stream: tcp_stream.try_clone()?, phantom: PhantomData})
    }
}

impl<InputType> ChannelDrivenThread<()> for TcpOutput<InputType>
    where InputType: Input {

    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl<InputType> Consumer<InputMessage<InputType>> for Sender<TcpOutput<InputType>>
    where InputType: Input {

    fn accept(&self, input_message: InputMessage<InputType>) {
        self.send(move |tcp_output|{

            let message = ToServerMessage::Input(input_message);
            rmp_serde::encode::write(&mut tcp_output.tcp_stream, &message).unwrap();

        }).unwrap();
    }
}