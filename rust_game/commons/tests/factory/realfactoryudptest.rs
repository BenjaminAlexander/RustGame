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

use commons::{
    factory::{
        FactoryTrait,
        RealFactory,
    },
    logging::LoggingConfigBuilder,
    net::UdpSocketTrait,
    threading::{
        AsyncJoin,
        SingleThreadExecutor,
    },
};
use log::{
    info,
    LevelFilter,
};

const A_NUMBER: u8 = 42;

struct TestStruct {
    received_number: Option<u8>,
}

#[test]
fn test_real_factory_tcp() {
    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);

    let test_struct = TestStruct {
        received_number: None,
    };

    let test_struct = Arc::new(Mutex::new(test_struct));

    let real_factory = RealFactory::new();

    let socket_addr_v4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0);
    let socket_addr = SocketAddr::from(socket_addr_v4);

    let udp_socket_1 = real_factory.bind_udp_socket(socket_addr).unwrap();
    let local_addr1 = udp_socket_1.local_addr().unwrap();

    let executor = SingleThreadExecutor::new();

    info!("Bound to local addr: {:?}", local_addr1);

    let test_struct_clone = test_struct.clone();
    let executor_clone = executor.clone();
    let udp_read_handler = move |peer_addr: SocketAddr, buf: &[u8]| {
        test_struct_clone.lock().unwrap().received_number = Some(buf[0]);

        executor_clone.stop();

        return ControlFlow::Continue(());
    };

    let _sender = real_factory
        .new_thread_builder()
        .name("UdpReader")
        .spawn_udp_reader(udp_socket_1, udp_read_handler, AsyncJoin::log_async_join)
        .unwrap();

    let mut udp_socket_2 = real_factory.bind_udp_socket(socket_addr).unwrap();

    udp_socket_2.send_to(&[A_NUMBER], &local_addr1).unwrap();

    executor.wait_for_join();

    let test_struct_guard = test_struct.lock().unwrap();

    assert_eq!(A_NUMBER, test_struct_guard.received_number.unwrap());
}
