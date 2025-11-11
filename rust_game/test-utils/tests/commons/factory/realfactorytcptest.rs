use commons::net::{
    LOCAL_EPHEMERAL_SOCKET_ADDR_V4, NET_POLLING_PERIOD, TcpListenerBuilder, TcpReadHandlerBuilder
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

    let mut tcp_connection_handler = TcpConnectionHandler::new();

    tcp_connection_handler.set_on_bind(move |socket_addr| {
        info!("TcpListener bound to {:?}", socket_addr);

        let (mut tcp_stream, _) = RealFactory::new().connect_tcp(socket_addr).unwrap();

        tcp_stream.write(&A_NUMBER).unwrap();
        tcp_stream.flush().unwrap();
    });

    let tcp_listener_builder = TcpListenerBuilder::new(&real_factory);

    let tcp_listener_stopper = tcp_listener_builder.get_stopper().clone();
    let expect_one_tcp_connection = async_expects.new_async_expect("Expect one TCP connection", ());
    let async_expects_clone = async_expects.clone();
    let mut tcp_reader_senders = Vec::new();
    tcp_connection_handler.set_on_connection(move |tcp_stream, tcp_reader| {
        expect_one_tcp_connection.set_actual(());

        let listener_remote_socket_addr = *tcp_stream.get_peer_addr();
        info!(
            "Connected Listened remote {:?}",
            listener_remote_socket_addr
        );

        let tcp_listener_stopper_clone = tcp_listener_stopper.clone();
        let expected_number_clone = expected_number.clone();
        let mut tcp_read_handler = TcpReadHandler::new(move |number: i32| {
            info!("Read a number {:?}", number);

            expected_number_clone.set_actual(number);

            //Stop the listener
            tcp_listener_stopper_clone.send_stop_thread().unwrap();

            return ControlFlow::Continue(());
        });

        let expect_tcp_read_error = async_expects_clone.new_async_expect("Expect TcpReader Join", ());
        tcp_read_handler.set_on_read_error(move ||{
            expect_tcp_read_error.set_actual(());
        });

        let real_factory = RealFactory::new();

        let sender = TcpReadHandlerBuilder::new_thread(
            &real_factory,
            "TcpReader".to_string(),
            tcp_reader,
            tcp_read_handler,
        )
        .unwrap();

        tcp_reader_senders.push(sender);

        return ControlFlow::Continue(());
    });

    let expect_tcp_listener_stop = async_expects.new_async_expect("Expect listener stop", ());
    tcp_connection_handler.set_on_stop(move ||{
        expect_tcp_listener_stop.set_actual(());
    });

    tcp_listener_builder
        .spawn_thread(
            "TcpListener".to_string(),
            SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4),
            tcp_connection_handler,
        )
        .unwrap();

    async_expects.wait_for_all();
}

#[test]
fn test_tcp_listener_channel_disconnect() {
    setup_test_logging();

    let async_expects = AsyncExpects::new();
    let real_factory = RealFactory::new();

    let mut tcp_connection_handler = TcpConnectionHandler::new();

    let expect_tcp_listener_channel_disconnect = async_expects.new_async_expect("Expect listener channel disconnect", ());
    tcp_connection_handler.set_on_channel_disconnected(move ||{
        expect_tcp_listener_channel_disconnect.set_actual(());
    });

    //Drop the sender to cause a channel disconnect
    TcpListenerBuilder::new_thread(
        &real_factory,
        "TcpListener".to_string(),
        SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4),
        tcp_connection_handler,
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

    let mut tcp_connection_handler = TcpConnectionHandler::new();

    tcp_connection_handler.set_on_bind(move |socket_addr| {
        executor.execute_function_or_panic(move || {
            //Sleep to cause the listener to poll
            std::thread::sleep(NET_POLLING_PERIOD.mul_f64(2.0).to_duration().unwrap());

            RealFactory::new().connect_tcp(socket_addr).unwrap();
        });
    });

    tcp_connection_handler.set_on_connection(move |_, _| {
        expect_connection.set_actual(());
        return ControlFlow::Break(());
    });

    let real_factory = RealFactory::new();

    let _sender = TcpListenerBuilder::new_thread(
        &real_factory,
        "TcpListener".to_string(),
        SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4),
        tcp_connection_handler,
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

    let real_factory = RealFactory::new();

    let thread_builder = TcpListenerBuilder::new(&real_factory);

    let mut tcp_connection_handler = TcpConnectionHandler::new();

    tcp_connection_handler.set_on_bind(move |socket_addr| {
        executor.execute_function_or_panic(move || {
            RealFactory::new().connect_tcp(socket_addr).unwrap();
        });
    });

    let sender = thread_builder.get_stopper().clone();
    tcp_connection_handler.set_on_connection(move |_, _| {
        expect_connection.set_actual(());
        sender.send_stop_thread().unwrap();
        return ControlFlow::Continue(());
    });

    let expect_tcp_listener_stop = async_expects.new_async_expect("Expect listener stop", ());
    tcp_connection_handler.set_on_stop(move ||{
        expect_tcp_listener_stop.set_actual(());
    });

    let _sender = thread_builder
        .spawn_thread(
            "TcpListener".to_string(),
            SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4),
            tcp_connection_handler,
        )
        .unwrap();

    async_expects.wait_for_all();
}

#[test]
fn test_stop_tcp_reader() {
    setup_test_logging();

    let async_expects = AsyncExpects::new();

    let real_factory = RealFactory::new();

    let mut tcp_connection_handler = TcpConnectionHandler::new();

    let async_expects_clone = async_expects.clone();
    let mut tcp_reader_senders = Vec::new();
    tcp_connection_handler.set_on_bind(move |socket_addr| {
        info!("TcpListener bound to {:?}", socket_addr);

        let (_, tcp_reader) = RealFactory::new().connect_tcp(socket_addr).unwrap();

        let mut tcp_read_handler = TcpReadHandler::new(move |_: i32| {
            return ControlFlow::Continue(());
        });

        let expecte_read_error = async_expects_clone.new_async_expect("TcpReader-ConnectorSide", ());
        tcp_read_handler.set_on_read_error(move || expecte_read_error.set_actual(()));

        let real_factory = RealFactory::new();

        let sender = TcpReadHandlerBuilder::new_thread(
            &real_factory,
            "TcpReader-ConnectorSide".to_string(),
            tcp_reader,
            tcp_read_handler,
        )
        .unwrap();

        tcp_reader_senders.push(sender);
    });

    let tcp_listener_builder = TcpListenerBuilder::new(&real_factory);

    let listener_sender = tcp_listener_builder.get_stopper().clone();
    let expect_one_tcp_connection = async_expects.new_async_expect("Expect one TCP connection", ());
    let async_expects_clone = async_expects.clone();
    tcp_connection_handler.set_on_connection(move |tcp_stream, tcp_reader| {
        expect_one_tcp_connection.set_actual(());

        let listener_remote_socket_addr = *tcp_stream.get_peer_addr();
        info!(
            "Connected Listened remote {:?}",
            listener_remote_socket_addr
        );

        let mut tcp_read_handler = TcpReadHandler::new(move |_: i32| {
            return ControlFlow::Continue(());
        });

        let expect_join =
            async_expects_clone.new_async_expect("Expect TcpReader-ListenerSide Stop", ());
        let listener_sender = listener_sender.clone();

        tcp_read_handler.set_on_stop(move ||{
            expect_join.set_actual(());
            listener_sender.send_stop_thread().unwrap();
        });

        let real_factory = RealFactory::new();

        let reader_sender = TcpReadHandlerBuilder::new_thread(
            &real_factory,
            "TcpReader-ListenerSide".to_string(),
            tcp_reader,
            tcp_read_handler,
        )
        .unwrap();

        //Sleep to cause the reader to poll
        std::thread::sleep(NET_POLLING_PERIOD.mul_f64(2.0).to_duration().unwrap());

        reader_sender.send_stop_thread().unwrap();

        return ControlFlow::Continue(());
    });

    let expect_tcp_listener_stop = async_expects.new_async_expect("Expect listener stop", ());
    tcp_connection_handler.set_on_stop(move ||{
        expect_tcp_listener_stop.set_actual(());
    });

    tcp_listener_builder
        .spawn_thread(
            "TcpListener".to_string(),
            SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4),
            tcp_connection_handler,
        )
        .unwrap();

    async_expects.wait_for_all();
}

#[test]
fn test_tcp_reader_channel_disconnect() {
    setup_test_logging();

    let async_expects = AsyncExpects::new();

    let real_factory = RealFactory::new();

    let mut tcp_connection_handler = TcpConnectionHandler::new();

    let async_expects_clone = async_expects.clone();
    tcp_connection_handler.set_on_bind(move |socket_addr| {
        info!("TcpListener bound to {:?}", socket_addr);

        let (_, tcp_reader) = RealFactory::new().connect_tcp(socket_addr).unwrap();

        let mut tcp_read_handler = TcpReadHandler::new(move |_: i32| {
            return ControlFlow::Continue(());
        });

        let expect_channel_disconnect = async_expects_clone.new_async_expect("TcpReader-ConnectorSide channel disconnect", ());
        tcp_read_handler.set_on_channel_disconnected(move ||expect_channel_disconnect.set_actual(()));

        let real_factory = RealFactory::new();

        // Drop the sender so the reader gets a channel disconnect
        TcpReadHandlerBuilder::new_thread(
            &real_factory,
            "TcpReader-ConnectorSide".to_string(),
            tcp_reader,
            tcp_read_handler,
        )
        .unwrap();
    });

    let expect_one_tcp_connection = async_expects.new_async_expect("Expect one TCP connection", ());
    let async_expects_clone = async_expects.clone();
    tcp_connection_handler.set_on_connection(move |tcp_stream, tcp_reader| {
        expect_one_tcp_connection.set_actual(());

        let listener_remote_socket_addr = *tcp_stream.get_peer_addr();
        info!(
            "Connected Listened remote {:?}",
            listener_remote_socket_addr
        );

        let mut tcp_read_handler = TcpReadHandler::new(move |_: i32| {
            return ControlFlow::Continue(());
        });

        let expect_channel_disconnect = async_expects_clone.new_async_expect("TcpReader-ListenerSide channel disconnect", ());
        tcp_read_handler.set_on_channel_disconnected(move ||expect_channel_disconnect.set_actual(()));

        let real_factory = RealFactory::new();

        // Drop the sender so the reader gets a channel disconnect
        TcpReadHandlerBuilder::new_thread(
            &real_factory,
            "TcpReader-ListenerSide".to_string(),
            tcp_reader,
            tcp_read_handler,
        )
        .unwrap();

        return ControlFlow::Break(());
    });

    let _sender = TcpListenerBuilder::new_thread(
        &real_factory,
        "TcpListener".to_string(),
        SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4),
        tcp_connection_handler,
    )
    .unwrap();

    async_expects.wait_for_all();
}
