use std::marker::PhantomData;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream};

use log::{error, info};
use crate::interface::Game;
use crate::server::ServerCore;

use crate::threading::{Consumer, Sender, ChannelThread, Receiver};
use crate::threading::sender::SendError;

pub struct TcpListenerThread<GameType: Game> {
    server_core_sender: Sender<ServerCore<GameType>>,
    phantom: PhantomData<GameType>
}

impl<GameType: Game> TcpListenerThread<GameType> {
    pub fn new(server_core_sender: Sender<ServerCore<GameType>>) -> Self {
        Self{server_core_sender, phantom: PhantomData}
    }
}

impl<GameType: Game> ChannelThread<()> for TcpListenerThread<GameType> {

    fn run(mut self, receiver: Receiver<Self>) {
        let socket_addr_v4:SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), GameType::TCP_PORT);
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