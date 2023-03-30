use std::ops::ControlFlow;
use crate::net::{TcpReceiverTrait, TcpSenderTrait};

pub trait TcpConnectionHandlerTrait: Send + 'static {
    type TcpSender: TcpSenderTrait;
    type TcpReceiver: TcpReceiverTrait;

    fn on_connection(&mut self, tcp_sender: Self::TcpSender, tcp_receiver: Self::TcpReceiver) -> ControlFlow<()>;
}