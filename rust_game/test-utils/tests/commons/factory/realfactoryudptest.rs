use commons::{
    factory::{
        FactoryTrait,
        RealFactory,
    },
    logging::LoggingConfigBuilder,
    net::UdpReadHandlerBuilder,
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

const A_NUMBER: u8 = 42;

#[test]
fn test_real_factory_udp() {
    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);

    let async_expects = AsyncExpects::new();

    let expected_number =
        async_expects.new_async_expect("An expected number sent over UDP", A_NUMBER);

    let real_factory = RealFactory::new();

    let mut send_udp_socket = real_factory.bind_udp_ephemeral_port().unwrap();

    let expected_socket = async_expects.new_async_expect(
        "The expected source socket",
        send_udp_socket.local_addr().unwrap(),
    );

    let udp_socket_1 = real_factory.bind_udp_ephemeral_port().unwrap();

    let local_addr1 = udp_socket_1.local_addr().unwrap();

    info!("Bound to local addr: {:?}", local_addr1);

    let expected_number_clone = expected_number.clone();
    let udp_read_handler = move |peer_addr: SocketAddr, buf: &[u8]| {
        expected_number_clone.set_actual(buf[0]);
        expected_socket.set_actual(peer_addr);

        return ControlFlow::Continue(());
    };

    let udp_socket_clone = udp_socket_1.try_clone().unwrap();

    let udp_reader_sender = UdpReadHandlerBuilder::new_thread(
        &real_factory,
        "UdpReader".to_string(),
        udp_socket_clone,
        udp_read_handler,
        async_expects.new_expect_async_join("UdpReader Join"),
    )
    .unwrap();

    send_udp_socket.send_to(&[A_NUMBER], &local_addr1).unwrap();

    expected_number.wait_for();

    udp_reader_sender.send_stop_thread().unwrap();

    async_expects.wait_for_all();
}

#[test]
fn test_udp_reader_break() {
    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);

    let async_expects = AsyncExpects::new();

    let expected_number =
        async_expects.new_async_expect("An expected number sent over UDP", A_NUMBER);

    let real_factory = RealFactory::new();

    let mut send_udp_socket = real_factory.bind_udp_ephemeral_port().unwrap();

    let udp_socket_1 = real_factory.bind_udp_ephemeral_port().unwrap();

    let local_addr1 = udp_socket_1.local_addr().unwrap();

    info!("Bound to local addr: {:?}", local_addr1);

    let udp_read_handler = move |_, buf: &[u8]| {
        expected_number.set_actual(buf[0]);

        return ControlFlow::Break(());
    };

    let _sender = UdpReadHandlerBuilder::new_thread(
        &real_factory,
        "UdpReader".to_string(),
        udp_socket_1,
        udp_read_handler,
        async_expects.new_expect_async_join("UdpReader Join"),
    )
    .unwrap();

    send_udp_socket.send_to(&[A_NUMBER], &local_addr1).unwrap();

    async_expects.wait_for_all();
}

#[test]
fn test_drop_udp_reader_sender() {
    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);

    let async_expects = AsyncExpects::new();

    let real_factory = RealFactory::new();

    let udp_socket_1 = real_factory.bind_udp_ephemeral_port().unwrap();

    let udp_read_handler = move |_, _: &[u8]| {
        panic!();
    };

    //Drop the sender
    UdpReadHandlerBuilder::new_thread(
        &real_factory,
        "UdpReader".to_string(),
        udp_socket_1,
        udp_read_handler,
        async_expects.new_expect_async_join("UdpReader Join"),
    )
    .unwrap();

    async_expects.wait_for_all();
}
