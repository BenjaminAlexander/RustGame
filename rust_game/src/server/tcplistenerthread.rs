use std::ops::ControlFlow::*;
use log::{error, info, warn};
use commons::factory::FactoryTrait;
use commons::ip::TcpListenerTrait;
use crate::interface::{GameFactoryTrait, ServerToClientTcpStream};
use crate::server::servercore::ServerCoreEvent;
use crate::server::servercore::ServerCoreEvent::TcpConnectionEvent;
use commons::threading::channel::ReceiveMetaData;
use commons::threading::eventhandling::{Sender, EventSenderTrait};
use commons::threading::listener::{ListenedOrDidNotListen, ChannelEvent, ListenerEventResult, ListenerTrait, ListenResult};
use commons::ip::TcpStreamTrait;

pub struct TcpListenerThread<GameFactory: GameFactoryTrait> {
    factory: GameFactory::Factory,
    tcp_listener: <GameFactory::Factory as FactoryTrait>::TcpListener,
    server_core_sender: Sender<GameFactory::Factory, ServerCoreEvent<GameFactory>>
}

impl<GameFactory: GameFactoryTrait> TcpListenerThread<GameFactory> {
    pub fn new(
        factory: GameFactory::Factory,
        server_core_sender: Sender<GameFactory::Factory, ServerCoreEvent<GameFactory>>,
        tcp_listener: <GameFactory::Factory as FactoryTrait>::TcpListener
    ) -> Self {
        return Self {
            factory,
            tcp_listener,
            server_core_sender
        };
    }

    fn handle_tcp_stream_and_socket_addr(self, tcp_stream: ServerToClientTcpStream<GameFactory>) -> ListenerEventResult<Self> {

        info!("New TCP connection from {:?}", tcp_stream.get_peer_addr());

        let stream_clone = match tcp_stream.try_clone() {
            Ok(stream_clone) => stream_clone,
            Err(error) => {
                error!("Unable to clone tcp stream: {:?}", error);
                return Continue(self);
            }
        };

        match self.server_core_sender.send_event(TcpConnectionEvent(stream_clone)) {
            Ok(()) => {
                return Continue(self);
            }
            Err(error) => {
                error!("Error sending to the core: {:?}", error);
                return Break(());
            }
        }
    }
}

impl<GameFactory: GameFactoryTrait> ListenerTrait for TcpListenerThread<GameFactory> {
    type Event = ();
    type ThreadReturn = ();
    type ListenFor = ServerToClientTcpStream<GameFactory>;

    fn listen(mut self) -> ListenResult<Self> {

        return match self.tcp_listener.accept() {
            Ok(tcp_stream) =>
                Continue(ListenedOrDidNotListen::Listened(self, tcp_stream)),
            Err(error) => {
                error!("Error on TcpListener.accept: {:?}", error);
                Continue(ListenedOrDidNotListen::DidNotListen(self))
            }
        }
    }

    fn on_channel_event(self, event: ChannelEvent<Self>) -> ListenerEventResult<Self> {
        return match event {
            ChannelEvent::ChannelEmptyAfterListen(_, value) => self.handle_tcp_stream_and_socket_addr(value),
            ChannelEvent::ReceivedEvent(_, ()) => {
                warn!("This listener doesn't have meaningful messages, but one was sent.");
                Continue(self)
            }
            ChannelEvent::ChannelDisconnected => Break(())
        }
    }

    fn on_stop(self, _: ReceiveMetaData) -> Self::ThreadReturn { () }
}