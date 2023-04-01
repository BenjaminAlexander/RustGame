use std::ops::ControlFlow;
use crate::net::{TcpReaderTrait, TcpSenderTrait};

pub trait TcpConnectionHandlerTrait: Send + 'static {
    type TcpSender: TcpSenderTrait;
    type TcpReceiver: TcpReaderTrait;

    fn on_connection(&mut self, tcp_sender: Self::TcpSender, tcp_receiver: Self::TcpReceiver) -> ControlFlow<()>;
}