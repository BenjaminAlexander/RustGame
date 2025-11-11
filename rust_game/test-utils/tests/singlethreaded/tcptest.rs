use commons::factory::FactoryTrait;
use commons::logging::LoggingConfigBuilder;
use commons::net::{
    TcpConnectionHandlerTrait,
    TcpListenerBuilder,
    TcpReadHandlerBuilder,
    TcpReadHandlerTrait,
    TcpReader,
    TcpStream,
};
use commons::single_threaded_simulator::SingleThreadedFactory;
use commons::threading::eventhandling::EventHandlerStopper;
use log::{
    error,
    info,
    LevelFilter,
};
use std::net::{
    IpAddr,
    Ipv4Addr,
    SocketAddr,
};
use std::ops::ControlFlow;
use std::ops::ControlFlow::Continue;
use std::sync::{
    Arc,
    Mutex,
};

const PORT: u16 = 1234;

#[test]
fn test_tcp() {
    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);

    let server_factory = SingleThreadedFactory::new();

    let server_side = Arc::new(Mutex::new(None));

    let connection_handler = ConnectionHandler {
        factory: server_factory.clone(),
        server_side: server_side.clone(),
    };

    let listen_socket = SocketAddr::new(server_factory.get_host_simulator().get_ip_addr(), PORT);

    let connection_handler_sender = TcpListenerBuilder::new_thread(
        &server_factory,
        "TcpConnectionListener".to_string(),
        listen_socket.clone(),
        connection_handler,
    )
    .unwrap();

    let client_factory = server_factory.clone_for_new_host(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)));

    let (tcp_stream, reader) = client_factory.connect_tcp(listen_socket).unwrap();

    let client_thread_builder = TcpReadHandlerBuilder::new(&client_factory);

    let client_side = Arc::new(Mutex::new(Some(TestConnection {
        tcp_stream,
        reader_sender: client_thread_builder.get_stopper().clone(),
        last_value: None,
    })));

    let client_read_handler = TcpReadHandler {
        test_connection: client_side.clone(),
    };

    let client_read_sender = client_thread_builder
        .spawn_thread(
            "TcpReader".to_string(),
            reader,
            client_read_handler,
            |_|{},
        )
        .unwrap();

    server_factory.get_time_queue().run_events();

    test_write(&server_factory, &server_side, &client_side, 1);
    test_write(&server_factory, &client_side, &server_side, 2);

    let send_result = connection_handler_sender.send_stop_thread();
    if send_result.is_err() {
        panic!("Send Failed")
    }
    server_factory.get_time_queue().run_events();

    test_write(&server_factory, &server_side, &client_side, 3);
    test_write(&server_factory, &client_side, &server_side, 4);

    let send_result = client_read_sender.send_stop_thread();
    if send_result.is_err() {
        panic!("Send Failed")
    }
    server_factory.get_time_queue().run_events();

    //test_write(&server_factory, &server_side, &client_side, 5);
    test_write(&server_factory, &client_side, &server_side, 6);
}

fn test_write(
    factory: &SingleThreadedFactory,
    write_connection: &Arc<Mutex<Option<TestConnection>>>,
    read_connection: &Arc<Mutex<Option<TestConnection>>>,
    value: u32,
) {
    {
        let mut guard = write_connection.lock().unwrap();

        guard.as_mut().unwrap().tcp_stream.write(&value).unwrap();
        guard.as_mut().unwrap().tcp_stream.flush().unwrap();
    }

    factory.get_time_queue().run_events();

    assert_eq!(
        value,
        read_connection
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .last_value
            .unwrap()
    );
}

struct TestConnection {
    tcp_stream: TcpStream,
    #[allow(dead_code)]
    reader_sender: EventHandlerStopper,
    last_value: Option<u32>,
}

struct ConnectionHandler {
    factory: SingleThreadedFactory,
    server_side: Arc<Mutex<Option<TestConnection>>>,
}

impl TcpConnectionHandlerTrait for ConnectionHandler {
    fn on_connection(&mut self, tcp_stream: TcpStream, tcp_receiver: TcpReader) -> ControlFlow<()> {
        info!(
            "{:?} is handling a connection from {:?}",
            self.factory.get_host_simulator().get_ip_addr(),
            tcp_stream.get_peer_addr()
        );

        let tcp_read_handler = TcpReadHandler {
            test_connection: self.server_side.clone(),
        };

        let reader_sender = TcpReadHandlerBuilder::new_thread(
            &self.factory,
            "TcpReader".to_string(),
            tcp_receiver,
            tcp_read_handler,
        )
        .unwrap();

        let server_side = TestConnection {
            tcp_stream,
            reader_sender,
            last_value: None,
        };

        let mut guard = self.server_side.lock().unwrap();

        if guard.is_some() {
            error!("Expected None");
            panic!("Expected None");
        }

        *guard = Some(server_side);

        return Continue(());
    }
}

struct TcpReadHandler {
    test_connection: Arc<Mutex<Option<TestConnection>>>,
}

impl TcpReadHandlerTrait for TcpReadHandler {
    type ReadType = u32;

    fn on_read(&mut self, read: Self::ReadType) -> ControlFlow<()> {
        if let Some(ref mut server_side) = *self.test_connection.lock().unwrap() {
            server_side.last_value = Some(read);
            return Continue(());
        } else {
            error!("Expected Some");
            panic!("Expected Some");
        }
    }
}
