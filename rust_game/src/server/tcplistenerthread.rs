use std::marker::PhantomData;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};

use log::{error, info};
use crate::interface::GameTrait;
use crate::server::ServerCore;

use crate::threading::{Sender, ChannelThread, Receiver};

pub struct TcpListenerThread<Game: GameTrait> {
    server_core_sender: Sender<ServerCore<Game>>,
    phantom: PhantomData<Game>
}

impl<Game: GameTrait> TcpListenerThread<Game> {
    pub fn new(server_core_sender: Sender<ServerCore<Game>>) -> Self {
        Self{server_core_sender, phantom: PhantomData}
    }
}

impl<Game: GameTrait> ChannelThread<()> for TcpListenerThread<Game> {

    fn run(mut self, receiver: Receiver<Self>) {
        let socket_addr_v4:SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), Game::TCP_PORT);
        let socket_addr:SocketAddr = SocketAddr::from(socket_addr_v4);
        let listener:TcpListener = TcpListener::bind(socket_addr).unwrap();

        // accept connections and process them serially
        for result in listener.incoming() {
            match result {
                Ok(tcp_stream) => {
                    info!("New TCP connection from {:?}", tcp_stream.peer_addr().unwrap().ip().to_string());
                    //core.addTcpStream(tcpStream);

                    //TODO: this doesn't really do anything, should probably check if listening should stop
                    receiver.try_iter(&mut self);

                    self.server_core_sender.on_tcp_connection(tcp_stream);
                }
                Err(error) => {
                    error!("{:?}", error);
                    return;
                }
            }
        };
    }
}