use crate::server::ServerCore;
use crate::GameTrait;
use commons::real_time::net::tcp::{
    HandleTcpConnection,
    TcpReader,
    TcpStream,
};
use log::{
    info,
    warn,
};
use std::ops::ControlFlow;
use std::ops::ControlFlow::*;

pub struct TcpConnectionHandler<Game: GameTrait> {
    server_core: ServerCore<Game>,
}

impl<Game: GameTrait> TcpConnectionHandler<Game> {
    pub fn new(server_core: ServerCore<Game>) -> Self {
        return Self { server_core };
    }
}

impl<Game: GameTrait> HandleTcpConnection for TcpConnectionHandler<Game> {
    fn on_connection(&mut self, tcp_stream: TcpStream, tcp_reader: TcpReader) -> ControlFlow<()> {
        info!("New TCP connection from {:?}", tcp_stream.get_peer_addr());

        match self
            .server_core
            .handle_tcp_connection(tcp_stream, tcp_reader)
        {
            Ok(_) => Continue(()),
            Err(_) => {
                warn!("Error sending TcpConnectionEvent to the Core");
                Break(())
            }
        }
    }
}
