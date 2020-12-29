use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream};

use log::{error, info};

use crate::threading::{Consumer, Thread};

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
        let socket_addr_v4:SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), self.port);
        let socket_addr:SocketAddr = SocketAddr::from(socket_addr_v4);
        let listener:TcpListener = TcpListener::bind(socket_addr).unwrap();

        // accept connections and process them serially
        for result in listener.incoming() {
            match result {
                Ok(tcp_stream) => {
                    info!("New TCP connection from {:?}", tcp_stream.peer_addr().unwrap().ip().to_string());
                    //core.addTcpStream(tcpStream);

                    self.consumer.accept(tcp_stream);
                }
                Err(error) => {
                    error!("{:?}", error);
                    return;
                }
            }
        };
    }
}