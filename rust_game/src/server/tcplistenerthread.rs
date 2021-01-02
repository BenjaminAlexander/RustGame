use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream};

use log::{error, info};

use crate::threading::{Consumer, Thread, ChannelDrivenThread, Sender, ChannelThread, Receiver};
use crate::threading::sender::SendError;

pub struct TcpListenerThread {
    port: u16,
    consumer: Option<Box<dyn Consumer<TcpStream>>>
}

impl TcpListenerThread {
    pub fn new(port:u16) -> TcpListenerThread {
        TcpListenerThread{port, consumer: None}
    }
}

impl ChannelThread<()> for TcpListenerThread {

    fn run(mut self, receiver: Receiver<Self>) {
        let socket_addr_v4:SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), self.port);
        let socket_addr:SocketAddr = SocketAddr::from(socket_addr_v4);
        let listener:TcpListener = TcpListener::bind(socket_addr).unwrap();

        // accept connections and process them serially
        for result in listener.incoming() {
            match result {
                Ok(tcp_stream) => {
                    info!("New TCP connection from {:?}", tcp_stream.peer_addr().unwrap().ip().to_string());
                    //core.addTcpStream(tcpStream);

                    receiver.try_iter(&mut self);

                    if self.consumer.is_some() {

                        self.consumer.as_ref().unwrap().accept(tcp_stream);
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

impl Sender<TcpListenerThread> {
    pub fn set_consumer<T>(&self, t: T) -> Result<(), SendError<TcpListenerThread>>
        where T: Consumer<TcpStream> {

        self.send(|tcp_listener_thread|{
            tcp_listener_thread.consumer = Some(Box::new(t));
        })
    }
}