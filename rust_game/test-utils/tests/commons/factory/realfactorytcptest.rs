use commons::logging::LoggingConfigBuilder;
use commons::net::LOCAL_EPHEMERAL_SOCKET_ADDR_V4;
use commons::threading::channel::RealSender;
use commons::threading::eventhandling::{
    EventOrStopThread,
    EventSenderTrait,
};
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
    threading::AsyncJoin,
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
use test_utils::assert::{
    AsyncExpect,
    AsyncExpects,
};

const A_NUMBER: i32 = 42;

struct TestStruct {
    async_expects: AsyncExpects,
    tcp_reader_sender: Option<RealSender<RealFactory, EventOrStopThread<()>>>,
    expected_socket: Option<AsyncExpect<SocketAddr>>,
}

#[test]
fn test_real_factory_tcp() {
    //TODO: write a set up logging method for tests
    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);

    let async_expects = AsyncExpects::new();

    let test_struct = TestStruct {
        async_expects: async_expects.clone(),
        tcp_reader_sender: None,
        expected_socket: None,
    };

    let expected_number =
        async_expects.new_async_expect("An expected number sent over TCP", A_NUMBER);

    let test_struct = Arc::new(Mutex::new(test_struct));

    let real_factory = RealFactory::new();

    let mut tcp_connection_handler = TcpConnectionHandler::<RealFactory>::new();

    let test_struct_clone = test_struct.clone();
    tcp_connection_handler.set_on_bind(move |socket_addr| {
        info!("TcpListener bound to {:?}", socket_addr);

        let (mut tcp_stream, _) = RealFactory::new().connect_tcp(socket_addr).unwrap();

        let connector_local_socket_addr = tcp_stream.local_addr().unwrap();
        info!("Connected local {:?}", connector_local_socket_addr);

        {
            let mut guard = test_struct_clone.lock().unwrap();
            let expected_socket = guard
                .async_expects
                .new_async_expect("Expected Socket", connector_local_socket_addr);
            guard.expected_socket = Some(expected_socket);
        }

        tcp_stream.write(&A_NUMBER).unwrap();
        tcp_stream.flush().unwrap();
    });

    let tcp_listener_builder = real_factory
        .new_thread_builder()
        .name("TcpListener")
        .build_channel_for_tcp_listener::<TcpConnectionHandler<RealFactory>>(
    );

    let tcp_listener_sender = tcp_listener_builder.clone_sender();

    let test_struct_clone = test_struct.clone();
    tcp_connection_handler.set_on_connection(move |tcp_stream, _| {
        let listener_remote_socket_addr = *tcp_stream.get_peer_addr();
        info!(
            "Connected Listened remote {:?}",
            listener_remote_socket_addr
        );

        test_struct
            .lock()
            .unwrap()
            .expected_socket
            .as_ref()
            .unwrap()
            .set_actual(listener_remote_socket_addr);

        let tcp_listener_sender_clone = tcp_listener_sender.clone();
        let expected_number_clone = expected_number.clone();
        let tcp_read_handler = TcpReadHandler::new(move |number: i32| {
            info!("Read a number {:?}", number);

            expected_number_clone.set_actual(number);

            //Stop the listener
            tcp_listener_sender_clone.send_stop_thread().unwrap();

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

    let expect_join = async_expects.new_async_expect("", ());

    let on_join = move |_| {
        expect_join.set_actual(());
    };

    tcp_listener_builder
        .spawn_tcp_listener(
            SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4),
            tcp_connection_handler,
            on_join,
        )
        .unwrap();

    async_expects.wait_for_all();

    let test_struct_guard = test_struct_clone.lock().unwrap();

    assert!(test_struct_guard.expected_socket.is_some());

    info!("Done");
}
