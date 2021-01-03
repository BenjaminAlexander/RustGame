use std::net::TcpStream;
use crate::gametime::TimeMessage;
use crate::threading::{ChannelDrivenThread, Consumer, Sender};
use std::io;
use crate::messaging::ToClientMessage;
use std::io::Write;

pub struct TcpOutput {
    tcp_stream: TcpStream,
    time_message: Option<TimeMessage>
}

impl ChannelDrivenThread<()> for TcpOutput {
    fn on_none_pending(&mut self) -> Option<()> {
        match self.time_message {
            None => {}
            Some(time_message) => {
                self.time_message = None;
                let message = ToClientMessage::TimeMessage(time_message);

                rmp_serde::encode::write(&mut self.tcp_stream, &message).unwrap();
                self.tcp_stream.flush().unwrap();
            }
        }
        None
    }

    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl TcpOutput {
    pub fn new(tcp_stream: &TcpStream) -> io::Result<TcpOutput> {
        Ok(TcpOutput{tcp_stream: tcp_stream.try_clone()?, time_message: None})
    }
}

impl Consumer<TimeMessage> for Sender<TcpOutput> {
    fn accept(&self, time_message: TimeMessage) {
        self.send(move |tcp_output|{
            if tcp_output.time_message.is_none() ||
                time_message.is_after(&tcp_output.time_message.clone().unwrap()) {
                tcp_output.time_message = Some(time_message);
            }
        }).unwrap();
    }
}