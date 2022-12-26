use std::marker::PhantomData;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};
use std::sync::mpsc::TryRecvError;

use log::{error, info};
use crate::interface::GameTrait;
use crate::server::ServerCore;

use crate::threading::{ChannelDrivenThreadSender as Sender, ChannelThread, Receiver, ThreadAction};

pub struct TcpListenerThread<Game: GameTrait> {
    server_core_sender: Sender<ServerCore<Game>>,
    phantom: PhantomData<Game>
}

impl<Game: GameTrait> TcpListenerThread<Game> {
    pub fn new(server_core_sender: Sender<ServerCore<Game>>) -> Self {
        Self{server_core_sender, phantom: PhantomData}
    }
}

impl<Game: GameTrait> ChannelThread<(), ThreadAction> for TcpListenerThread<Game> {

    fn run(mut self, receiver: Receiver<Self, ThreadAction>) {
        let socket_addr_v4:SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), Game::TCP_PORT);
        let socket_addr:SocketAddr = SocketAddr::from(socket_addr_v4);


        let listener = match TcpListener::bind(socket_addr) {
            Ok(tcp_listener) => tcp_listener,
            Err(error) => {
                error!("{:?}", error);
                return;
            }
        };

        // accept connections and process them serially
        for result in listener.incoming() {
            match result {
                Ok(tcp_stream) => {

                    match tcp_stream.peer_addr() {
                        Ok(socket_addr) => {
                            info!("New TCP connection from {:?}", socket_addr.ip().to_string());
                        }
                        Err(error) => {
                            error!("Unable to get tcp stream peer address");
                            error!("{:?}", error);
                        }
                    }

                    loop {
                        match receiver.try_recv(&mut self) {
                            Ok(ThreadAction::Continue) => {}
                            Err(TryRecvError::Empty) => break,
                            Ok(ThreadAction::Stop) => {
                                info!("Thread commanded to stop.");
                                return;
                            }
                            Err(TryRecvError::Disconnected) => {
                                info!("Thread stopping due to disconnect.");
                                return;
                            }
                        }
                    }

                    match self.server_core_sender.on_tcp_connection(tcp_stream) {
                        Ok(_) => {/*contiue*/}
                        Err(error) => {
                            error!("{:?}", error);
                            return;
                        }
                    }
                }
                Err(error) => {
                    error!("{:?}", error);
                    return;
                }
            }
        };
    }
}