use commons::net::{
    LOCAL_EPHEMERAL_SOCKET_ADDR_V4,
    TCP_POLLING_PERIOD,
};
use commons::threading::channel::RealSender;
use commons::threading::eventhandling::{
    EventOrStopThread,
    EventSenderTrait,
};
use commons::threading::SingleThreadExecutor;
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
};
use log::info;
use std::{
    net::SocketAddr,
    ops::ControlFlow,
};
use test_utils::assert::AsyncExpects;
use test_utils::utils::setup_test_logging;

const A_NUMBER: i32 = 42;

#[test]
fn test_real_factory_tcp() {
    setup_test_logging();

    let async_expects = AsyncExpects::new();

    let expected_number =
        async_expects.new_async_expect("An expected number sent over TCP", A_NUMBER);

    let real_factory = RealFactory::new();

    let mut tcp_connection_handler = TcpConnectionHandler::<RealFactory>::new();

    tcp_connection_handler.set_on_bind(move |socket_addr| {
        info!("TcpListener bound to {:?}", socket_addr);

        let (mut tcp_stream, _) = RealFactory::new().connect_tcp(socket_addr).unwrap();

        let connector_local_socket_addr = tcp_stream.local_addr().unwrap();
        info!("Connected local {:?}", connector_local_socket_addr);

        tcp_stream.write(&A_NUMBER).unwrap();
        tcp_stream.flush().unwrap();
    });

    let tcp_listener_builder = real_factory
        .new_thread_builder()
        .name("TcpListener")
        .build_channel_for_tcp_listener::<TcpConnectionHandler<RealFactory>>(
    );

    let tcp_listener_sender = tcp_listener_builder.clone_sender();
    let expect_one_tcp_connection = async_expects.new_async_expect("Expect one TCP connection", ());
    let async_expects_clone = async_expects.clone();
    let mut tcp_reader_senders = Vec::new();
    tcp_connection_handler.set_on_connection(move |tcp_stream, _| {
        expect_one_tcp_connection.set_actual(());

        let listener_remote_socket_addr = *tcp_stream.get_peer_addr();
        info!(
            "Connected Listened remote {:?}",
            listener_remote_socket_addr
        );

        let tcp_listener_sender_clone = tcp_listener_sender.clone();
        let expected_number_clone = expected_number.clone();
        let tcp_read_handler = TcpReadHandler::new(move |number: i32| {
            info!("Read a number {:?}", number);

            expected_number_clone.set_actual(number);

            //Stop the listener
            tcp_listener_sender_clone.send_stop_thread().unwrap();

            return ControlFlow::Continue(());
        });

        let sender: RealSender<RealFactory, EventOrStopThread<()>> = RealFactory::new()
            .new_thread_builder()
            .name("TcpReader")
            .spawn_tcp_reader(
                tcp_stream,
                tcp_read_handler,
                async_expects_clone.new_expect_async_join("Expect TcpReader Join"),
            )
            .unwrap();

        tcp_reader_senders.push(sender);

        return ControlFlow::Continue(());
    });

    tcp_listener_builder
        .spawn_tcp_listener(
            SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4),
            tcp_connection_handler,
            async_expects.new_expect_async_join("Expect listener join"),
        )
        .unwrap();

    async_expects.wait_for_all();
}

#[test]
fn test_tcp_listener_channel_disconnect() {
    setup_test_logging();

    let async_expects = AsyncExpects::new();

    //Drop the sender to cause a channel disconnect
    RealFactory::new()
        .new_thread_builder()
        .name("TcpListener")
        .spawn_tcp_listener(
            SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4),
            TcpConnectionHandler::<RealFactory>::new(),
            async_expects.new_expect_async_join("Expect listener join"),
        )
        .unwrap();

    async_expects.wait_for_all();
}

#[test]
fn test_tcp_listener_polling_timeout() {
    setup_test_logging();

    let async_expects = AsyncExpects::new();

    let expect_connection = async_expects.new_async_expect("Expect one TCP connection", ());

    let executor = SingleThreadExecutor::new();

    let mut tcp_connection_handler = TcpConnectionHandler::<RealFactory>::new();

    tcp_connection_handler.set_on_bind(move |socket_addr| {
        executor.execute_function_or_panic(move || {
            //Sleep to cause the listener to poll
            std::thread::sleep(
                TCP_POLLING_PERIOD
                    .mul_f64(2.0)
                    .to_duration()
                    .unwrap(),
            );

            RealFactory::new().connect_tcp(socket_addr).unwrap();
        });
    });

    tcp_connection_handler.set_on_connection(move |_, _| {
        expect_connection.set_actual(());
        return ControlFlow::Break(());
    });

    let _sender = RealFactory::new()
        .new_thread_builder()
        .name("TcpListener")
        .spawn_tcp_listener(
            SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4),
            tcp_connection_handler,
            async_expects.new_expect_async_join("Expect listener join"),
        )
        .unwrap();

    async_expects.wait_for_all();
}

#[test]
fn test_tcp_listener_send_event() {
    setup_test_logging();

    let async_expects = AsyncExpects::new();

    let expect_connection = async_expects.new_async_expect("Expect one TCP connection", ());

    let executor = SingleThreadExecutor::new();

    let thread_builder = RealFactory::new()
        .new_thread_builder()
        .name("TcpListener")
        .build_channel_for_tcp_listener::<TcpConnectionHandler<RealFactory>>();

    let mut tcp_connection_handler = TcpConnectionHandler::<RealFactory>::new();

    let sender = thread_builder.clone_sender();
    tcp_connection_handler.set_on_bind(move |socket_addr| {
        let sender = sender.clone();
        executor.execute_function_or_panic(move || {
            sender.send_event(()).unwrap();

            RealFactory::new().connect_tcp(socket_addr).unwrap();
        });
    });

    let sender = thread_builder.clone_sender();
    tcp_connection_handler.set_on_connection(move |_, _| {
        expect_connection.set_actual(());
        sender.send_stop_thread().unwrap();
        return ControlFlow::Continue(());
    });

    let _sender = thread_builder
        .spawn_tcp_listener(
            SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4),
            tcp_connection_handler,
            async_expects.new_expect_async_join("Expect listener join"),
        )
        .unwrap();

    async_expects.wait_for_all();
}

#[test]
fn test_stop_tcp_reader() {
    setup_test_logging();

    let async_expects = AsyncExpects::new();

    let real_factory = RealFactory::new();

    let mut tcp_connection_handler = TcpConnectionHandler::<RealFactory>::new();

    let async_expects_clone = async_expects.clone();
    let mut tcp_reader_senders = Vec::new();
    tcp_connection_handler.set_on_bind(move |socket_addr| {
        info!("TcpListener bound to {:?}", socket_addr);

        let (tcp_stream, _) = RealFactory::new().connect_tcp(socket_addr).unwrap();

        let tcp_read_handler = TcpReadHandler::new(move |_: i32| {
            return ControlFlow::Continue(());
        });

        let sender: RealSender<RealFactory, EventOrStopThread<()>> = RealFactory::new()
            .new_thread_builder()
            .name("TcpReader-ConnectorSide")
            .spawn_tcp_reader(
                tcp_stream,
                tcp_read_handler,
                async_expects_clone.new_expect_async_join("TcpReader-ConnectorSide"),
            )
            .unwrap();

        tcp_reader_senders.push(sender);
    });

    let tcp_listener_builder = real_factory
        .new_thread_builder()
        .name("TcpListener")
        .build_channel_for_tcp_listener::<TcpConnectionHandler<RealFactory>>(
    );

    let listener_sender = tcp_listener_builder.clone_sender();
    let expect_one_tcp_connection = async_expects.new_async_expect("Expect one TCP connection", ());
    let async_expects_clone = async_expects.clone();
    tcp_connection_handler.set_on_connection(move |tcp_stream, _| {
        expect_one_tcp_connection.set_actual(());

        let listener_remote_socket_addr = *tcp_stream.get_peer_addr();
        info!(
            "Connected Listened remote {:?}",
            listener_remote_socket_addr
        );

        let tcp_read_handler = TcpReadHandler::new(move |_: i32| {
            return ControlFlow::Continue(());
        });

        let tcp_stream_clone = tcp_stream.try_clone().unwrap();

        let reader_sender: RealSender<RealFactory, EventOrStopThread<()>> = RealFactory::new()
            .new_thread_builder()
            .name("TcpReader-ListenerSide")
            .spawn_tcp_reader(
                tcp_stream_clone,
                tcp_read_handler,
                async_expects_clone.new_expect_async_join("Expect TcpReader-ListenerSide Join"),
            )
            .unwrap();

        //Sleep to cause the reader to poll
        std::thread::sleep(
            TCP_POLLING_PERIOD
                .mul_f64(2.0)
                .to_duration()
                .unwrap(),
        );

        listener_sender.send_stop_thread().unwrap();
        reader_sender.send_stop_thread().unwrap();

        return ControlFlow::Continue(());
    });

    tcp_listener_builder
        .spawn_tcp_listener(
            SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4),
            tcp_connection_handler,
            async_expects.new_expect_async_join("Expect listener join"),
        )
        .unwrap();

    async_expects.wait_for_all();
}
