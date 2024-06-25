use commons::logging::LoggingConfigBuilder;
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
use core::panic;
use log::{
    info,
    LevelFilter,
};
use std::{
    net::{
        Ipv4Addr,
        SocketAddr,
        SocketAddrV4,
    },
    ops::ControlFlow,
    sync::{
        Arc,
        Mutex,
    },
};

const A_NUMBER: i32 = 42;

#[test]
fn test_real_factory_tcp() {
    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);

    let tcp_reader_sender = Arc::new(Mutex::new(Option::None));
    let received_number = Arc::new(Mutex::new(Option::None));

    let executor = SingleThreadExecutor::new();

    let real_factory = RealFactory::new();

    let socket_addr_v4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0);
    let socket_addr = SocketAddr::from(socket_addr_v4);

    let mut tcp_connection_handler = TcpConnectionHandler::<RealFactory>::new();

    tcp_connection_handler.set_on_bind(|socket_addr| {
        info!("TcpListener bound to {:?}", socket_addr);

        let (mut tcp_stream, _) = RealFactory::new().connect_tcp(socket_addr).unwrap();

        info!("Connected local {:?}", tcp_stream.local_addr().unwrap());

        tcp_stream.write(&A_NUMBER).unwrap();
        tcp_stream.flush().unwrap();
    });

    let executor_clone = executor.clone();
    let tcp_reader_sender_clone = tcp_reader_sender.clone();
    let received_number_clone = received_number.clone();
    tcp_connection_handler.set_on_connection(move |tcp_stream, _| {
        info!("Connected Listened remote {:?}", tcp_stream.get_peer_addr());

        let executor = executor_clone.clone();
        let received_number_clone = received_number_clone.clone();
        let tcp_read_handler = TcpReadHandler::new(move |number: i32| {
            info!("Read a number {:?}", number);
            *received_number_clone.lock().unwrap() = Some(number);

            let executor_clone = executor.clone();

            executor
                .execute_function(move || {
                    executor_clone.stop();
                })
                .unwrap_or_else(|_| panic!("Failed to send function"));

            return ControlFlow::Continue(());
        });

        let sender = RealFactory::new()
            .new_thread_builder()
            .name("TcpReader")
            .spawn_tcp_reader(tcp_stream, tcp_read_handler, AsyncJoin::log_async_join)
            .unwrap();

        *tcp_reader_sender_clone.lock().unwrap() = Some(sender);

        return ControlFlow::Continue(());
    });

    let _sender = real_factory
        .new_thread_builder()
        .name("TcpListener")
        .spawn_tcp_listener(
            socket_addr,
            tcp_connection_handler,
            AsyncJoin::log_async_join,
        )
        .unwrap();

    executor.wait_for_join();

    assert_eq!(A_NUMBER, received_number.lock().unwrap().unwrap());

    info!("Done");
}
