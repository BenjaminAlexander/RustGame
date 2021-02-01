use log::{trace, info};
use std::net::TcpStream;
use crate::threading::{ChannelDrivenThread, Consumer, Sender, ChannelThread, Receiver};
use crate::interface::{Input, InputEvent};
use crate::messaging::{InputMessage, ToServerMessage};
use std::io;
use std::marker::PhantomData;
use std::io::Write;

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

impl<InputType, InputEventType> ChannelThread<()> for TcpOutput<InputType, InputEventType>
    where InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    fn run(mut self, receiver: Receiver<Self>) -> () {
        loop {
            trace!("Waiting.");
            match receiver.recv(&mut self) {
                Err(_error) => {
                    info!("Channel closed.");
                    return ();
                }
                _ => {}
            }

            let mut send_another_message = true;
            while send_another_message {
                receiver.try_iter(&mut self);

                match self.input_queue.pop() {
                    None => send_another_message = false,
                    Some(input_to_send) => {
                        let message = ToServerMessage::<InputType>::Input(input_to_send);
                        rmp_serde::encode::write(&mut self.tcp_stream, &message).unwrap();
                        self.tcp_stream.flush().unwrap();
                    }
                }

            }
        }
    }
}

impl<InputType, InputEventType> Consumer<InputMessage<InputType>> for Sender<TcpOutput<InputType, InputEventType>>
    where InputType: Input<InputEventType>,
          InputEventType: InputEvent {

    fn accept(&self, input_message: InputMessage<InputType>) {
        self.send(move |tcp_output|{

            //insert in reverse sorted order
            match tcp_output.input_queue.binary_search_by(|elem| { input_message.cmp(elem) }) {
                Ok(pos) => tcp_output.input_queue[pos] = input_message,
                Err(pos) => tcp_output.input_queue.insert(pos, input_message)
            }

        }).unwrap();
    }
}