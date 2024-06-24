use crate::interface::{
    EventSender,
    GameFactoryTrait,
    TcpReader,
    TcpWriter,
};
use crate::server::servercore::ServerCoreEvent;
use crate::server::servercore::ServerCoreEvent::TcpConnectionEvent;
use commons::net::{
    TcpConnectionHandlerTrait,
    TcpWriterTrait,
};
use commons::threading::eventhandling::EventSenderTrait;
use log::{
    info,
    warn,
};
use std::ops::ControlFlow;
use std::ops::ControlFlow::*;

pub struct TcpConnectionHandler<GameFactory: GameFactoryTrait> {
    server_core_sender: EventSender<GameFactory, ServerCoreEvent<GameFactory>>,
}

impl<GameFactory: GameFactoryTrait> TcpConnectionHandler<GameFactory> {
    pub fn new(server_core_sender: EventSender<GameFactory, ServerCoreEvent<GameFactory>>) -> Self {
        return Self { server_core_sender };
    }
}

impl<GameFactory: GameFactoryTrait> TcpConnectionHandlerTrait
    for TcpConnectionHandler<GameFactory>
{
    type Factory = GameFactory::Factory;

    fn on_connection(
        &mut self,
        tcp_sender: TcpWriter<GameFactory>,
        tcp_receiver: TcpReader<GameFactory>,
    ) -> ControlFlow<()> {
        info!("New TCP connection from {:?}", tcp_sender.get_peer_addr());

        let send_result = self
            .server_core_sender
            .send_event(TcpConnectionEvent(tcp_sender, tcp_receiver));

        return match send_result {
            Ok(_) => Continue(()),
            Err(_) => {
                warn!("Error sending TcpConnectionEvent to the Core");
                Break(())
            }
        };
    }
}
