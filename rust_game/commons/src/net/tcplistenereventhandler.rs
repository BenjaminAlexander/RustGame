use std::net::TcpListener;
use std::ops::ControlFlow::{Break, Continue};
use log::error;
use crate::factory::RealFactory;
use crate::net::realtcpstream::RealTcpStream;
use crate::net::tcpconnectionhandlertrait::TcpConnectionHandlerTrait;
use crate::net::TcpWriterTrait;
use crate::threading::channel::ReceiveMetaData;
use crate::threading::eventhandling::{ChannelEvent, ChannelEventResult, EventHandlerTrait};
use crate::threading::eventhandling::WaitOrTryForNextEvent::TryForNextEvent;

pub struct TcpListenerEventHandler<T: TcpConnectionHandlerTrait<Factory=RealFactory>> {
    tcp_listener: TcpListener,
    tcp_connection_handler: T

}

impl<T: TcpConnectionHandlerTrait<Factory=RealFactory>> TcpListenerEventHandler<T> {

    pub fn new(tcp_listener: TcpListener, tcp_connection_handler: T) -> Self {
        return Self {
            tcp_listener,
            tcp_connection_handler
        };
    }

    fn accept(mut self) -> ChannelEventResult<Self> {

        match self.tcp_listener.accept() {
            Ok((tcp_stream, remote_peer_socket_addr)) => {

                let tcp_stream = RealTcpStream::new(tcp_stream, remote_peer_socket_addr);
                if let Ok(tcp_stream_clone) = tcp_stream.try_clone() {

                    match self.tcp_connection_handler.on_connection(tcp_stream, tcp_stream_clone) {
                        Continue(()) => {
                            return Continue(TryForNextEvent(self));
                        }
                        Break(()) => {
                            return Break(self.tcp_connection_handler);
                        }
                    }
                } else {
                    error!("Failed to clone RealTcpStream for : {:?}", tcp_stream.get_peer_addr());
                    return Continue(TryForNextEvent(self));
                }

            }
            Err(error) => {
                error!("Error while trying to accept a TCP connection: {:?}", error);
                return Continue(TryForNextEvent(self));
            }
        }
    }
}

impl<T: TcpConnectionHandlerTrait<Factory=RealFactory>> EventHandlerTrait for TcpListenerEventHandler<T> {
    type Event = ();
    type ThreadReturn = T;

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> ChannelEventResult<Self> {
        return match channel_event {
            ChannelEvent::ReceivedEvent(_, ()) => Continue(TryForNextEvent(self)),
            ChannelEvent::Timeout => Continue(TryForNextEvent(self)),
            ChannelEvent::ChannelEmpty => self.accept(),
            ChannelEvent::ChannelDisconnected => Break(self.tcp_connection_handler)
        };
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        return self.tcp_connection_handler;
    }
}