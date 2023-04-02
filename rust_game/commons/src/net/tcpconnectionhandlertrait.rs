use std::ops::ControlFlow;
use crate::net::{TcpReaderTrait, TcpWriterTrait};

pub trait TcpConnectionHandlerTrait: Send + 'static {
    //TODO: replace with factory
    type TcpSender: TcpWriterTrait;
    type TcpReceiver: TcpReaderTrait;

    fn on_connection(&mut self, tcp_sender: Self::TcpSender, tcp_receiver: Self::TcpReceiver) -> ControlFlow<()>;
}