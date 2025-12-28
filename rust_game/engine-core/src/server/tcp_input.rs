use {
    crate::messaging::ToServerMessageTCP,
    commons::real_time::{
        EventHandlerStopper,
        Factory,
        net::tcp::{
            HandleTcpRead,
            TcpReadHandlerBuilder,
            TcpReader,
        },
    },
    std::{
        io::Error,
        ops::ControlFlow,
    },
};

pub struct TcpInput {
    stopper: EventHandlerStopper,
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

        Ok(TcpInput { stopper })
    }

    pub fn stop(&self) -> Result<(), ()> {
        self.stopper.send_stop_thread()
    }
}

pub struct ReadHandler;

impl HandleTcpRead for ReadHandler {
    type ReadType = ToServerMessageTCP;

    fn on_read(&mut self, read: Self::ReadType) -> ControlFlow<()> {
        match read {};
    }
}
