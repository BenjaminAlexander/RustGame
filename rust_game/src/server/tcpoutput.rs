use std::net::TcpStream;
use crate::gametime::TimeMessage;
use crate::threading::{ChannelDrivenThread, Consumer, Sender};
use std::io;
use crate::messaging::{ToClientMessage, InputMessage};
use std::io::Write;
use crate::interface::Input;

pub struct TcpOutput<InputType>
    where InputType: Input {

    tcp_stream: TcpStream,
    time_message: Option<TimeMessage>,
    input_queue: Vec<InputMessage<InputType>>
}

impl<InputType> ChannelDrivenThread<()> for TcpOutput<InputType>
    where InputType: Input {

    fn on_none_pending(&mut self) -> Option<()> {

        if self.time_message.is_some() {
            let time_message = self.time_message.unwrap();
            self.time_message = None;

            let message = ToClientMessage::<InputType>::TimeMessage(time_message);

            rmp_serde::encode::write(&mut self.tcp_stream, &message).unwrap();
            self.tcp_stream.flush().unwrap();

        } else if !self.input_queue.is_empty() {
            match self.input_queue.pop() {
                None => {}
                Some(input_to_send) => {

                    let message = ToClientMessage::InputMessage(input_to_send);
                    rmp_serde::encode::write(&mut self.tcp_stream, &message).unwrap();
                    self.tcp_stream.flush().unwrap();
                }
            }
        }

        None
    }

    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl<InputType> TcpOutput<InputType>
    where InputType: Input {

    pub fn new(tcp_stream: &TcpStream) -> io::Result<Self> {
        Ok(TcpOutput{
            tcp_stream: tcp_stream.try_clone()?,
            time_message: None,
            input_queue: Vec::new()
        })
    }
}

impl<InputType> Consumer<TimeMessage> for Sender<TcpOutput<InputType>>
    where InputType: Input {

    fn accept(&self, time_message: TimeMessage) {
        self.send(move |tcp_output|{
            if tcp_output.time_message.is_none() ||
                time_message.is_after(&tcp_output.time_message.clone().unwrap()) {
                tcp_output.time_message = Some(time_message);
            }
        }).unwrap();
    }
}

impl<InputType> Consumer<InputMessage<InputType>> for Sender<TcpOutput<InputType>>
    where InputType: Input {

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