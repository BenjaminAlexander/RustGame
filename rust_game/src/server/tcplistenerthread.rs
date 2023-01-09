use std::marker::PhantomData;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream};
use std::ops::ControlFlow::*;
use log::{error, info, warn};
use crate::interface::GameTrait;
use crate::server::ServerCore;
use crate::threading::ChannelDrivenThreadSender;
use crate::threading::listener::{ListenedOrDidNotListen, ListenedValueHolder, ChannelEvent, ListenerEventResult, ListenerTrait, ListenResult};

pub struct TcpListenerThread<Game: GameTrait> {
    tcp_listener_option: Option<TcpListener>,
    server_core_sender: ChannelDrivenThreadSender<ServerCore<Game>>,
    phantom: PhantomData<Game>
}

impl<Game: GameTrait> TcpListenerThread<Game> {
    pub fn new(server_core_sender: ChannelDrivenThreadSender<ServerCore<Game>>) -> Self {
        Self{
            tcp_listener_option: None,
            server_core_sender,
            phantom: PhantomData
        }
    }

    fn handle_tcp_stream_and_socket_addr(self, heard_value: ListenedValueHolder<Self>) -> ListenerEventResult<Self> {

        let (tcp_stream, socket_addr) = heard_value.get_value();

        info!("First Adder {:?}", socket_addr.to_string());

        match tcp_stream.peer_addr() {
            Ok(socket_addr) => {
                info!("New TCP connection from {:?}", socket_addr.to_string());
            }
            Err(error) => {
                error!("Unable to get tcp stream peer address");
                error!("{:?}", error);
            }
        }

        let stream_clone = match tcp_stream.try_clone() {
            Ok(stream_clone) => stream_clone,
            Err(error) => {
                error!("Unable to clone tcp stream: {:?}", error);
                return Continue(self);
            }
        };

        match self.server_core_sender.on_tcp_connection(stream_clone) {
            Ok(()) => {
                return Continue(self);
            }
            Err(error) => {
                error!("Error sending to the core: {:?}", error);
                return Break(self.on_stop());
            }
        }
    }
}

impl<Game: GameTrait> ListenerTrait for TcpListenerThread<Game> {
    type Event = ();
    type ThreadReturn = ();
    type ListenFor = (TcpStream, SocketAddr);

    fn listen(mut self) -> ListenResult<Self> {

        let tcp_listner = match self.tcp_listener_option.as_ref() {
            None => {
                let socket_addr_v4:SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), Game::TCP_PORT);
                let socket_addr:SocketAddr = SocketAddr::from(socket_addr_v4);

                self.tcp_listener_option = Some(match TcpListener::bind(socket_addr) {
                    Ok(tcp_listener) => tcp_listener,
                    Err(error) => {
                        error!("Error while binding TcpListener: {:?}", error);
                        return Break(self.on_stop());
                    }
                });

                self.tcp_listener_option.as_ref().unwrap()
            }
            Some(tcp_listner) => tcp_listner
        };

        return match tcp_listner.accept() {
            Ok(tcp_stream_and_socket_addr) =>
                Continue(ListenedOrDidNotListen::Listened(self, tcp_stream_and_socket_addr)),
            Err(error) => {
                error!("Error on TcpListener.accept: {:?}", error);
                Continue(ListenedOrDidNotListen::DidNotListen(self))
            }
        }
    }

    fn on_channel_event(self, event: ChannelEvent<Self>) -> ListenerEventResult<Self> {
        return match event {
            ChannelEvent::ChannelEmptyAfterListen(heard_value) => self.handle_tcp_stream_and_socket_addr(heard_value),
            ChannelEvent::ReceivedEvent(received_event_holder) =>
                match received_event_holder.move_event() {
                    () => {
                        warn!("This listener doesn't have meaningful messages, but one was sent.");
                        Continue(self)
                    }
                }
            ChannelEvent::ChannelDisconnected => Break(self.on_stop())
        }
    }

    fn on_stop(self) -> Self::ThreadReturn { () }
}