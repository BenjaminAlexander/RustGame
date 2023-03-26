use std::io::Error;
use std::net::{TcpListener, ToSocketAddrs};

pub struct RealTcpListener {
    tcp_listener: TcpListener
}

impl RealTcpListener {
    pub fn bind(socket_addr: impl ToSocketAddrs) -> Result<Self, Error>{
        return Ok(Self {
            tcp_listener: TcpListener::bind(socket_addr)?
        });
    }
}