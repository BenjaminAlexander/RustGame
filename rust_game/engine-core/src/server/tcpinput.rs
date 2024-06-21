use crate::messaging::ToServerMessageTCP;
use commons::net::TcpReadHandlerTrait;
use std::ops::ControlFlow;
use std::ops::ControlFlow::Continue;

pub struct TcpInput {}

impl TcpInput {
    pub fn new() -> Self {
        return Self {};
    }
}

impl TcpReadHandlerTrait for TcpInput {
    type ReadType = ToServerMessageTCP;

    fn on_read(&mut self, read: Self::ReadType) -> ControlFlow<()> {
        match read {};

        return Continue(());
    }
}
