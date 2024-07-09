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
};
use log::{
    info,
    LevelFilter,
};
use std::{
    net::SocketAddr,
    ops::ControlFlow,
};
use test_utils::assert::AsyncExpects;

const A_NUMBER: i32 = 42;

#[test]
fn test_real_factory_tcp() {
    //TODO: write a set up logging method for tests
    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);

    let async_expects = AsyncExpects::new();

    let expected_number =
        async_expects.new_async_expect("An expected number sent over TCP", A_NUMBER);

    let real_factory = RealFactory::new();

    let mut tcp_connection_handler = TcpConnectionHandler::<RealFactory>::new();

    let expect_one_tcp_connection = async_expects.new_async_expect("Expect one TCP connection", ());
    tcp_connection_handler.set_on_bind(move |socket_addr| {
        info!("TcpListener bound to {:?}", socket_addr);

        let (mut tcp_stream, _) = RealFactory::new().connect_tcp(socket_addr).unwrap();

        let connector_local_socket_addr = tcp_stream.local_addr().unwrap();
        info!("Connected local {:?}", connector_local_socket_addr);

        expect_one_tcp_connection.set_actual(());

        tcp_stream.write(&A_NUMBER).unwrap();
        tcp_stream.flush().unwrap();
    });

    let tcp_listener_builder = real_factory
        .new_thread_builder()
        .name("TcpListener")
        .build_channel_for_tcp_listener::<TcpConnectionHandler<RealFactory>>(
    );

    let tcp_listener_sender = tcp_listener_builder.clone_sender();

    let async_expects_clone = async_expects.clone();
    let mut tcp_reader_senders = Vec::new();
    tcp_connection_handler.set_on_connection(move |tcp_stream, _| {
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
    //TODO: write a set up logging method for tests
    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);

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
