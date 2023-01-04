use std::marker::PhantomData;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream};
use log::{error, info};
use crate::interface::GameTrait;
use crate::server::ServerCore;

use crate::threading::{ListenResult, ChannelDrivenThreadSender as Sender, ListenedValue, ListenerEventResult, ListenerEvent, ListenerTrait};

pub struct TcpListenerThread<Game: GameTrait> {
    tcp_listener_option: Option<TcpListener>,
    server_core_sender: Sender<ServerCore<Game>>,
    phantom: PhantomData<Game>
}

impl<Game: GameTrait> TcpListenerThread<Game> {
    pub fn new(server_core_sender: Sender<ServerCore<Game>>) -> Self {
        Self{
            tcp_listener_option: None,
            server_core_sender,
            phantom: PhantomData
        }
    }

    fn handle_tcp_stream_and_socket_addr(self, heard_value: ListenedValue<Self>) -> ListenerEventResult<Self> {

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
                error!("Unable to get clone tcp stream: {:?}", error);
                return ListenerEventResult::Continue(self);
            }
        };

        match self.server_core_sender.on_tcp_connection(stream_clone) {
            Ok(()) => {
                return ListenerEventResult::Continue(self);
            }
            Err(error) => {
                error!("Error sending to the core: {:?}", error);
                return ListenerEventResult::Stop(self.on_stop());
            }
        }
    }
}

impl<Game: GameTrait> ListenerTrait for TcpListenerThread<Game> {
    type MessageType = ();
    type ThreadReturnType = ();
    type ListenForType = (TcpStream, SocketAddr);

    fn listen(mut self) -> ListenResult<Self> {

        let tcp_listner = match self.tcp_listener_option.as_ref() {
            None => {
                let socket_addr_v4:SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), Game::TCP_PORT);
                let socket_addr:SocketAddr = SocketAddr::from(socket_addr_v4);

                self.tcp_listener_option = Some(match TcpListener::bind(socket_addr) {
                    Ok(tcp_listener) => tcp_listener,
                    Err(error) => {
                        error!("Error while binding TcpListener: {:?}", error);
                        return ListenResult::DidNotListen(ListenerEventResult::Stop(self.on_stop()));
                    }
                });

                self.tcp_listener_option.as_ref().unwrap()
            }
            Some(tcp_listner) => tcp_listner
        };

        return match tcp_listner.accept() {
            Ok(tcp_stream_and_socket_addr) =>
                ListenResult::Listened(self, tcp_stream_and_socket_addr),
            Err(error) => {
                error!("Error on TcpListener.accept: {:?}", error);
                ListenResult::DidNotListen(ListenerEventResult::Continue(self))
            }
        }
    }

    fn on_event(self, event: ListenerEvent<Self>) -> ListenerEventResult<Self> {
        return match event {
            ListenerEvent::ChannelEmptyAfterListen(heard_value) => self.handle_tcp_stream_and_socket_addr(heard_value),
            ListenerEvent::Message(()) => ListenerEventResult::Continue(self),
            ListenerEvent::ChannelDisconnected => ListenerEventResult::Stop(self.on_stop())
        }
    }

    fn on_stop(self) -> Self::ThreadReturnType {
        return ();
    }
}