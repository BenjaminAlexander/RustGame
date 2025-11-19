use crate::server::servercore::ServerCoreEvent;
use crate::server::servercore::ServerCoreEvent::TcpConnectionEvent;
use crate::GameTrait;
use commons::real_time::net::tcp::{
    HandleTcpConnection,
    TcpReader,
    TcpStream,
};
use commons::real_time::EventSender;
use log::{
    info,
    warn,
};
use std::ops::ControlFlow;
use std::ops::ControlFlow::*;

pub struct TcpConnectionHandler<Game: GameTrait> {
    server_core_sender: EventSender<ServerCoreEvent<Game>>,
}

impl<Game: GameTrait> TcpConnectionHandler<Game> {
    pub fn new(server_core_sender: EventSender<ServerCoreEvent<Game>>) -> Self {
        return Self { server_core_sender };
    }
}

impl<Game: GameTrait> HandleTcpConnection for TcpConnectionHandler<Game> {
    fn on_connection(&mut self, tcp_stream: TcpStream, tcp_receiver: TcpReader) -> ControlFlow<()> {
        info!("New TCP connection from {:?}", tcp_stream.get_peer_addr());

        let send_result = self
            .server_core_sender
            .send_event(TcpConnectionEvent(tcp_stream, tcp_receiver));

        return match send_result {
            Ok(_) => Continue(()),
            Err(_) => {
                warn!("Error sending TcpConnectionEvent to the Core");
                Break(())
            }
        };
    }
}
