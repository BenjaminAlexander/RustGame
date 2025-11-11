use commons::net::{
    TcpListenerBuilder,
    TcpReadHandlerBuilder,
    LOCAL_EPHEMERAL_SOCKET_ADDR_V4,
    NET_POLLING_PERIOD,
};
use commons::threading::SingleThreadExecutor;
use commons::{
    factory::RealFactory,
    net::{
        TcpConnectionHandler,
        TcpReadHandler,
    },
};
use log::info;
use serde::{
    Deserialize,
    Serialize,
};
use std::io::Write;
use std::net::TcpStream;
use std::{
    net::SocketAddr,
    ops::ControlFlow,
};
use test_utils::assert::AsyncExpects;
use test_utils::utils::setup_test_logging;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
struct StructToSend {
    number_a: i64,
    number_b: i64,
}

const EXPECTED_STRUCT: StructToSend = StructToSend {
    number_a: 23,
    number_b: 47,
};

#[test]
fn test_non_blocking_tcp_reader() {
    setup_test_logging();

    let async_expects = AsyncExpects::new();

    let real_factory = RealFactory::new();

    let executor = SingleThreadExecutor::new();

    let mut tcp_connection_handler = TcpConnectionHandler::new();

    let executor_clone = executor.clone();
    tcp_connection_handler.set_on_bind(move |socket_addr| {
        info!("TcpListener bound to {:?}", socket_addr);

        executor_clone.execute_function_or_panic(move || {
            info!("EXPECTED_STRUCT: {:?}", EXPECTED_STRUCT);
            let vec = rmp_serde::encode::to_vec(&EXPECTED_STRUCT).unwrap();

            info!("EXPECTED_STRUCT as vec: {:?}", vec);
            let mut tcp_stream = TcpStream::connect(socket_addr).unwrap();

            tcp_stream.write_all(&vec[0..1]).unwrap();
            tcp_stream.flush().unwrap();

            std::thread::sleep(NET_POLLING_PERIOD.mul_f64(3.0).to_duration().unwrap());

            tcp_stream.write_all(&vec[1..]).unwrap();
            tcp_stream.flush().unwrap();
        });
    });

    let tcp_listener_builder = TcpListenerBuilder::new(&real_factory);

    let listener_sender = tcp_listener_builder.get_stopper().clone();
    let expect_one_tcp_connection = async_expects.new_async_expect("Expect one TCP connection", ());
    let async_expects_clone = async_expects.clone();
    let expected_struct =
        async_expects.new_async_expect("A struct to send and receive", EXPECTED_STRUCT);
    let mut tcp_reader_senders = Vec::new();
    tcp_connection_handler.set_on_connection(move |_, tcp_reader| {
        expect_one_tcp_connection.set_actual(());

        let listener_sender = listener_sender.clone();
        let expected_struct = expected_struct.clone();
        let tcp_read_handler = TcpReadHandler::new(move |actual_struct: StructToSend| {
            expected_struct.set_actual(actual_struct);
            listener_sender.send_stop_thread().unwrap();
            return ControlFlow::Break(());
        });

        let real_factory = RealFactory::new();

        let reader_sender = TcpReadHandlerBuilder::new(&real_factory)
            .spawn_thread_with_call_back(
                "TcpReader-ListenerSide".to_string(),
                tcp_reader,
                tcp_read_handler,
                async_expects_clone.new_expect_async_join("Expect TcpReader-ListenerSide Join"),
            )
            .unwrap();

        tcp_reader_senders.push(reader_sender);

        return ControlFlow::Continue(());
    });

    tcp_listener_builder
        .spawn_thread_with_call_back(
            "TcpListener".to_string(),
            SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4),
            tcp_connection_handler,
            async_expects.new_expect_async_join("Expect listener join"),
        )
        .unwrap();

    async_expects.wait_for_all();

    executor.stop();
    executor.wait_for_join();
}
