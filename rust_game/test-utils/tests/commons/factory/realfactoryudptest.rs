use commons::{
    factory::{
        FactoryTrait,
        RealFactory,
    },
    logging::LoggingConfigBuilder,
    net::UdpSocketTrait,
    threading::AsyncJoin,
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
fn test_real_factory_tcp() {
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

    let udp_read_handler = move |peer_addr: SocketAddr, buf: &[u8]| {
        expected_number.set_actual(buf[0]);
        expected_socket.set_actual(peer_addr);

        return ControlFlow::Continue(());
    };

    let udp_socket_clone = udp_socket_1.try_clone().unwrap();

    let _sender = real_factory
        .new_thread_builder()
        .name("UdpReader")
        .spawn_udp_reader(
            udp_socket_clone,
            udp_read_handler,
            AsyncJoin::log_async_join,
        )
        .unwrap();

    send_udp_socket.send_to(&[A_NUMBER], &local_addr1).unwrap();

    async_expects.wait_for_all();

    assert_eq! {true, udp_socket_1.peer_addr().is_err()}
}
