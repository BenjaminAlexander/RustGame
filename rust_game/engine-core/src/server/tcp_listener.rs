use {
    crate::{
        Play,
        server::ServerCore,
    },
    commons::real_time::{
        EventHandlerStopper,
        Factory,
        net::tcp::{
            HandleTcpConnection,
            TcpListenerBuilder,
            TcpReader,
            TcpStream,
        },
    },
    log::{
        info,
        warn,
    },
    std::{
        io::Error,
        net::SocketAddr,
        ops::{
            ControlFlow,
            ControlFlow::*,
        },
    },
};

pub struct TcpListener {
    sender: EventHandlerStopper,
}

impl TcpListener {
    pub fn new<Game: Play>(
        factory: &Factory,
        socket_addr: SocketAddr,
        server_core: ServerCore<Game>,
    ) -> Result<Self, Error> {
        let sender = TcpListenerBuilder::new_thread(
            &factory,
            "ServerTcpListener".to_string(),
            socket_addr,
            TcpConnectionHandler::<Game>::new(server_core),
        )?;

        Ok(Self { sender })
    }

    pub fn stop(&self) -> Result<(), ()> {
        self.sender.send_stop_thread()
    }
}

struct TcpConnectionHandler<Game: Play> {
    server_core: ServerCore<Game>,
}

impl<Game: Play> TcpConnectionHandler<Game> {
    pub fn new(server_core: ServerCore<Game>) -> Self {
        return Self { server_core };
    }
}

impl<Game: Play> HandleTcpConnection for TcpConnectionHandler<Game> {
    fn on_connection(&mut self, tcp_stream: TcpStream, tcp_reader: TcpReader) -> ControlFlow<()> {
        info!("New TCP connection from {:?}", tcp_stream.get_peer_addr());

        match self
            .server_core
            .handle_tcp_connection(tcp_stream, tcp_reader)
        {
            Ok(_) => Continue(()),
            Err(_) => {
                warn!("Error sending TcpConnectionEvent to the Core");
                Break(())
            }
        }
    }
}
