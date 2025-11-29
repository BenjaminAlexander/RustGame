use commons::real_time::net::tcp::{
    HandleTcpRead,
    TcpReadHandlerBuilder,
    TcpReader,
};
use commons::real_time::{
    EventHandlerStopper,
    Factory,
};

use crate::messaging::ToServerMessageTCP;
use std::io::Error;
use std::ops::ControlFlow;

pub struct TcpInput {
    _stopper: EventHandlerStopper,
}

impl TcpInput {
    pub fn new(
        factory: &Factory,
        player_index: usize,
        tcp_reader: TcpReader,
    ) -> Result<Self, Error> {
        let stopper = TcpReadHandlerBuilder::new_thread(
            factory,
            format!("ServerTcpInput-Player-{}", player_index),
            tcp_reader,
            ReadHandler,
        )?;

        Ok(TcpInput { _stopper: stopper })
    }

}

pub struct ReadHandler;

impl HandleTcpRead for ReadHandler {
    type ReadType = ToServerMessageTCP;

    fn on_read(&mut self, read: Self::ReadType) -> ControlFlow<()> {
        match read {};
    }
}
