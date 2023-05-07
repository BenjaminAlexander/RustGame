use std::ops::ControlFlow;
use std::ops::ControlFlow::*;
use log::{error, info};
use commons::net::{TcpConnectionHandlerTrait, TcpWriterTrait};
use crate::interface::{EventSender, GameFactoryTrait, TcpReader, TcpWriter};
use crate::server::servercore::ServerCoreEvent;
use crate::server::servercore::ServerCoreEvent::TcpConnectionEvent;
use commons::threading::eventhandling::EventSenderTrait;

pub struct TcpConnectionHandler<GameFactory: GameFactoryTrait> {
    server_core_sender: EventSender<GameFactory, ServerCoreEvent<GameFactory>>
}

impl<GameFactory: GameFactoryTrait> TcpConnectionHandler<GameFactory> {
    pub fn new(server_core_sender: EventSender<GameFactory, ServerCoreEvent<GameFactory>>) -> Self {
        return Self {
            server_core_sender
        };
    }
}

impl<GameFactory: GameFactoryTrait> TcpConnectionHandlerTrait for TcpConnectionHandler<GameFactory> {
    type Factory = GameFactory::Factory;

    fn on_connection(&mut self, tcp_sender: TcpWriter<GameFactory>, tcp_receiver: TcpReader<GameFactory>) -> ControlFlow<()> {
        info!("New TCP connection from {:?}", tcp_sender.get_peer_addr());

        match self.server_core_sender.send_event(TcpConnectionEvent(tcp_sender, tcp_receiver)) {
            Ok(()) => {
                return Continue(());
            }
            Err(error) => {
                error!("Error sending to the core: {:?}", error);
                return Break(());
            }
        }
    }
}