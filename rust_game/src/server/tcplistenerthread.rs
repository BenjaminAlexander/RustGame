use std::ops::ControlFlow;
use std::ops::ControlFlow::*;
use log::{error, info, warn};
use commons::factory::FactoryTrait;
use commons::net::{TcpConnectionHandler, TcpListenerTrait};
use crate::interface::{GameFactoryTrait, TcpListener, TcpStream};
use crate::server::servercore::ServerCoreEvent;
use crate::server::servercore::ServerCoreEvent::TcpConnectionEvent;
use commons::threading::channel::ReceiveMetaData;
use commons::threading::eventhandling::{Sender, EventSenderTrait};
use commons::threading::listener::{ListenedOrDidNotListen, ChannelEvent, ListenerEventResult, ListenerTrait, ListenResult};
use commons::net::TcpStreamTrait;

//TODO: rename this
pub struct TcpListenerThread<GameFactory: GameFactoryTrait> {
    server_core_sender: Sender<GameFactory::Factory, ServerCoreEvent<GameFactory>>
}

impl<GameFactory: GameFactoryTrait> TcpListenerThread<GameFactory> {
    pub fn new(server_core_sender: Sender<GameFactory::Factory, ServerCoreEvent<GameFactory>>) -> Self {
        return Self {
            server_core_sender
        };
    }
}

impl<GameFactory: GameFactoryTrait> TcpConnectionHandler for TcpListenerThread<GameFactory> {
    type TcpStream = <<GameFactory::Factory as FactoryTrait>::TcpListener as TcpListenerTrait>::TcpStream;

    fn on_connection(&mut self, tcp_stream: Self::TcpStream) -> ControlFlow<()> {
        info!("New TCP connection from {:?}", tcp_stream.get_peer_addr());

        let stream_clone = match tcp_stream.try_clone() {
            Ok(stream_clone) => stream_clone,
            Err(error) => {
                error!("Unable to clone tcp stream: {:?}", error);
                return Continue(());
            }
        };

        match self.server_core_sender.send_event(TcpConnectionEvent(stream_clone)) {
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