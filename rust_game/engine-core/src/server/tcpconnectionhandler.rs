use crate::interface::GameFactoryTrait;
use crate::server::servercore::ServerCoreEvent;
use crate::server::servercore::ServerCoreEvent::TcpConnectionEvent;
use commons::net::{
    TcpConnectionHandlerTrait,
    TcpReader,
    TcpStream,
};
use commons::threading::eventhandling::EventSender;
use log::{
    info,
    warn,
};
use std::ops::ControlFlow;
use std::ops::ControlFlow::*;

pub struct TcpConnectionHandler<GameFactory: GameFactoryTrait> {
    server_core_sender: EventSender<ServerCoreEvent<GameFactory>>,
}

impl<GameFactory: GameFactoryTrait> TcpConnectionHandler<GameFactory> {
    pub fn new(server_core_sender: EventSender<ServerCoreEvent<GameFactory>>) -> Self {
        return Self { server_core_sender };
    }
}

impl<GameFactory: GameFactoryTrait> TcpConnectionHandlerTrait
    for TcpConnectionHandler<GameFactory>
{
    fn on_connection(
        self,
        tcp_stream: TcpStream,
        tcp_receiver: TcpReader,
    ) -> ControlFlow<(), Self> {
        info!("New TCP connection from {:?}", tcp_stream.get_peer_addr());

        let send_result = self
            .server_core_sender
            .send_event(TcpConnectionEvent(tcp_stream, tcp_receiver));

        return match send_result {
            Ok(_) => Continue(self),
            Err(_) => {
                warn!("Error sending TcpConnectionEvent to the Core");
                Break(())
            }
        };
    }
}
