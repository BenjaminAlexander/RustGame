use crate::threading::{Thread, Consumer};
use std::net::{TcpStream, SocketAddrV4, Ipv4Addr, TcpListener, SocketAddr};
use log::{info, warn, error};

pub struct TcpListenerThread<T: Consumer<TcpStream>> {
    port:u16,
    consumer: T
}

impl<T: Consumer<TcpStream>> TcpListenerThread<T> {
    pub fn new(port:u16, consumer: T) -> TcpListenerThread<T> {
        TcpListenerThread{port, consumer}
    }
}

impl<T: Consumer<TcpStream>> Thread<()> for TcpListenerThread<T> {
    fn run(self) -> () {
        let socketAddrV4:SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), self.port);
        let socketAddr:SocketAddr = SocketAddr::from(socketAddrV4);
        let listener:TcpListener = TcpListener::bind(socketAddr).unwrap();

        // accept connections and process them serially
        for result in listener.incoming() {
            match result {
                Ok(tcpStream) => {
                    info!("New TCP connection from {:?}", tcpStream.peer_addr().unwrap().ip().to_string());
                    //core.addTcpStream(tcpStream);

                    self.consumer.accept(tcpStream);
                },
                Err(error) => {
                    error!("{:?}", error);
                    return;
                }
            }
        };
    }
}