use std::{
    net::SocketAddr,
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
    peer_addr: Option<SocketAddr>,
}

#[test]
fn test_real_factory_tcp() {
    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);

    let test_struct = TestStruct {
        received_number: None,
        peer_addr: None,
    };

    let test_struct = Arc::new(Mutex::new(test_struct));

    let real_factory = RealFactory::new();

    let udp_socket_1 = real_factory.bind_udp_ephemeral_port().unwrap();

    let local_addr1 = udp_socket_1.local_addr().unwrap();

    let executor = SingleThreadExecutor::new();

    info!("Bound to local addr: {:?}", local_addr1);

    let test_struct_clone = test_struct.clone();
    let executor_clone = executor.clone();
    let udp_read_handler = move |peer_addr: SocketAddr, buf: &[u8]| {
        {
            let mut guard = test_struct_clone.lock().unwrap();
            guard.received_number = Some(buf[0]);
            guard.peer_addr = Some(peer_addr);
        }

        executor_clone.stop();

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

    let mut udp_socket_2 = real_factory.bind_udp_ephemeral_port().unwrap();

    udp_socket_2.send_to(&[A_NUMBER], &local_addr1).unwrap();

    executor.wait_for_join();

    let test_struct_guard = test_struct.lock().unwrap();

    assert_eq!(A_NUMBER, test_struct_guard.received_number.unwrap());
    assert_eq!(
        udp_socket_2.local_addr().unwrap(),
        test_struct_guard.peer_addr.unwrap()
    );

    assert_eq! {true, udp_socket_1.peer_addr().is_err()}
}
