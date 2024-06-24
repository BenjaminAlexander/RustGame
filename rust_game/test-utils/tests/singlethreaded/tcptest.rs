use commons::factory::FactoryTrait;
use commons::logging::LoggingConfigBuilder;
use commons::net::{
    TcpConnectionHandlerTrait,
    TcpReadHandlerTrait,
    TcpWriterTrait,
};
use commons::threading::eventhandling::{
    EventOrStopThread,
    EventSenderTrait,
};
use commons::threading::AsyncJoin;
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
use test_utils::net::ChannelTcpWriter;
use test_utils::singlethreaded::{
    SingleThreadedFactory,
    SingleThreadedReceiver,
    SingleThreadedSender,
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

    let connection_handler_sender = server_factory
        .new_thread_builder()
        .name("TcpConnectionListener")
        .spawn_tcp_listener(
            listen_socket.clone(),
            connection_handler,
            AsyncJoin::log_async_join,
        )
        .unwrap();

    let client_factory = server_factory.clone_for_new_host(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)));

    let (writer, reader) = client_factory.connect_tcp(listen_socket).unwrap();

    let client_thread_builder = client_factory
        .new_thread_builder()
        .build_channel_thread::<EventOrStopThread<()>>();

    let client_side = Arc::new(Mutex::new(Some(TestConnection {
        writer,
        reader_sender: client_thread_builder.get_sender().clone(),
        last_value: None,
    })));

    let client_read_handler = TcpReadHandler {
        test_connection: client_side.clone(),
    };

    let client_read_sender = client_thread_builder
        .spawn_tcp_reader(reader, client_read_handler, AsyncJoin::log_async_join)
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

        guard.as_mut().unwrap().writer.write(&value).unwrap();
        guard.as_mut().unwrap().writer.flush().unwrap();
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
    writer: ChannelTcpWriter,
    #[allow(dead_code)]
    reader_sender: SingleThreadedSender<EventOrStopThread<()>>,
    last_value: Option<u32>,
}

struct ConnectionHandler {
    factory: SingleThreadedFactory,
    server_side: Arc<Mutex<Option<TestConnection>>>,
}

impl TcpConnectionHandlerTrait<SingleThreadedFactory> for ConnectionHandler {
    fn on_connection(
        &mut self,
        tcp_sender: ChannelTcpWriter,
        tcp_receiver: SingleThreadedReceiver<Vec<u8>>,
    ) -> ControlFlow<()> {
        info!(
            "{:?} is handling a connection from {:?}",
            self.factory.get_host_simulator().get_ip_addr(),
            tcp_sender.get_peer_addr()
        );

        let tcp_read_handler = TcpReadHandler {
            test_connection: self.server_side.clone(),
        };

        let reader_sender = self
            .factory
            .new_thread_builder()
            .spawn_tcp_reader(tcp_receiver, tcp_read_handler, AsyncJoin::log_async_join)
            .unwrap();

        let server_side = TestConnection {
            writer: tcp_sender,
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
