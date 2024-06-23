use std::{clone, sync::{Arc, Mutex}};
use commons::{factory::{FactoryTrait, RealFactory}, threading::{channel, eventhandling::EventHandlerTrait}};
use log::{info, LevelFilter};
use commons::{logging::LoggingConfigBuilder, threading::eventhandling::{ChannelEvent, EventHandleResult, EventSenderTrait}};

struct TestEventHandler {

}

impl EventHandlerTrait for TestEventHandler {
    type Event = ();

    type ThreadReturn = ();

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> EventHandleResult<Self> {

        info!("Channel event");

        match channel_event {
            ChannelEvent::ReceivedEvent(_, _) => todo!(),
            ChannelEvent::Timeout => todo!(),
            ChannelEvent::ChannelEmpty => todo!(),
            ChannelEvent::ChannelDisconnected => todo!(),
        };
        
        return EventHandleResult::StopThread(());
    }

    fn on_stop(self, receive_meta_data: channel::ReceiveMetaData) -> Self::ThreadReturn {
        return ();
    }
}

#[test]
fn test_real_factory() {

    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);

    let real_factory = RealFactory::new();

    let event_handler = TestEventHandler{};

    let wait_for_join = Arc::new(Mutex::new(true));

    let wait_for_join_clone = wait_for_join.clone();

    let sender = real_factory.new_thread_builder()
        .name("TestThread")
        .spawn_event_handler(event_handler, move |_| {
            info!("Setting wait_for_join to false");
            *wait_for_join_clone.lock().unwrap() = false;
        }).unwrap();

        if sender.send_stop_thread().is_err() {
            panic!("Stop thread failed");
        }

    while *wait_for_join.lock().unwrap()  {};

    info!("Done");

    /* 
    let x = real_factory.new_thread_builder()
        .name(&"TestTcpListner")
        .spawn_tcp_listener(
            socket_addr, 
            tcp_connection_handler, 
            AsyncJoin::log_async_join
        );
*/

    //real_factory.spawn_tcp_listener(thread_builder, socket_addr, tcp_connection_handler, join_call_back)

    //assert_eq!(add(1, 2), 3);
}

/* 
struct ConnectionHandler {
    factory: RealFactory,
    server_side: Arc<Mutex<Option<TestConnection>>>,
}

impl TcpConnectionHandlerTrait for ConnectionHandler {
    type Factory = RealFactory;

    fn on_connection(
        &mut self,
        tcp_sender: ChannelTcpWriter,
        tcp_receiver: SingleThreadedReceiver<Vec<u8>>,
    ) -> ControlFlow<()> {
        info!(
            "{:?} is handling a connection from {:?}",
            self.factory.get_host_simulator().get_ip_addr(),
            tcp_sender.get_peer_addr()
        );

        let tcp_read_handler = TcpReadHandler {
            test_connection: self.server_side.clone(),
        };

        let reader_sender = self
            .factory
            .new_thread_builder()
            .spawn_tcp_reader(tcp_receiver, tcp_read_handler, AsyncJoin::log_async_join)
            .unwrap();

        let server_side = TestConnection {
            writer: tcp_sender,
            reader_sender,
            last_value: None,
        };

        let mut guard = self.server_side.lock().unwrap();

        if guard.is_some() {
            error!("Expected None");
            panic!("Expected None");
        }

        *guard = Some(server_side);

        return Continue(());
    }
}*/