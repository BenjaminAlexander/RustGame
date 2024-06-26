use commons::logging::LoggingConfigBuilder;
use commons::net::LOCAL_EPHEMERAL_SOCKET_ADDR_V4;
use commons::threading::channel::RealSender;
use commons::threading::eventhandling::EventOrStopThread;
use commons::{
    factory::{
        FactoryTrait,
        RealFactory,
    },
    net::{
        TcpConnectionHandler,
        TcpReadHandler,
        TcpWriterTrait,
    },
    threading::{
        AsyncJoin,
        SingleThreadExecutor,
    },
};
use log::{
    info,
    LevelFilter,
};
use std::{
    net::SocketAddr,
    ops::ControlFlow,
    sync::{
        Arc,
        Mutex,
    },
};

const A_NUMBER: i32 = 42;

struct TestStruct {
    tcp_reader_sender: Option<RealSender<RealFactory, EventOrStopThread<()>>>,
    received_number: Option<i32>,
    connector_local_socket_addr: Option<SocketAddr>,
    listener_remote_socket_addr: Option<SocketAddr>,
}

#[test]
fn test_real_factory_tcp() {
    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);

    let test_struct = TestStruct {
        tcp_reader_sender: None,
        received_number: None,
        connector_local_socket_addr: None,
        listener_remote_socket_addr: None,
    };

    let test_struct = Arc::new(Mutex::new(test_struct));

    let executor = SingleThreadExecutor::new();

    let real_factory = RealFactory::new();

    let mut tcp_connection_handler = TcpConnectionHandler::<RealFactory>::new();

    let test_struct_clone = test_struct.clone();
    tcp_connection_handler.set_on_bind(move |socket_addr| {
        info!("TcpListener bound to {:?}", socket_addr);

        let (mut tcp_stream, _) = RealFactory::new().connect_tcp(socket_addr).unwrap();

        let connector_local_socket_addr = tcp_stream.local_addr().unwrap();
        info!("Connected local {:?}", connector_local_socket_addr);

        test_struct_clone
            .lock()
            .unwrap()
            .connector_local_socket_addr = Some(connector_local_socket_addr);

        tcp_stream.write(&A_NUMBER).unwrap();
        tcp_stream.flush().unwrap();
    });

    let executor_clone = executor.clone();
    let test_struct_clone = test_struct.clone();
    tcp_connection_handler.set_on_connection(move |tcp_stream, _| {
        let listener_remote_socket_addr = *tcp_stream.get_peer_addr();
        info!(
            "Connected Listened remote {:?}",
            listener_remote_socket_addr
        );

        test_struct.lock().unwrap().listener_remote_socket_addr = Some(listener_remote_socket_addr);

        let executor = executor_clone.clone();
        let test_struct_clone = test_struct.clone();
        let tcp_read_handler = TcpReadHandler::new(move |number: i32| {
            info!("Read a number {:?}", number);
            test_struct_clone.lock().unwrap().received_number = Some(number);

            executor.stop();

            return ControlFlow::Continue(());
        });

        let sender = RealFactory::new()
            .new_thread_builder()
            .name("TcpReader")
            .spawn_tcp_reader(tcp_stream, tcp_read_handler, AsyncJoin::log_async_join)
            .unwrap();

        //Hold on to the sender so its not dropped
        test_struct.lock().unwrap().tcp_reader_sender = Some(sender);

        return ControlFlow::Continue(());
    });

    let _sender = real_factory
        .new_thread_builder()
        .name("TcpListener")
        .spawn_tcp_listener(
            SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4),
            tcp_connection_handler,
            AsyncJoin::log_async_join,
        )
        .unwrap();

    executor.wait_for_join();

    let test_struct_guard = test_struct_clone.lock().unwrap();

    assert_eq!(A_NUMBER, test_struct_guard.received_number.unwrap());

    assert_eq!(
        test_struct_guard.connector_local_socket_addr.unwrap(),
        test_struct_guard.listener_remote_socket_addr.unwrap()
    );

    info!("Done");
}
