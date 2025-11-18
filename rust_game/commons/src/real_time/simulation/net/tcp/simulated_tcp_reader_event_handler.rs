use crate::real_time::event_or_stop_thread::EventOrStopThread;
use crate::real_time::net::tcp::TcpReadHandlerTrait;
use crate::real_time::simulation::receiver_link::ReceiveOrDisconnected;
use crate::real_time::simulation::SingleThreadedReceiver;
use crate::real_time::{
    EventHandleResult,
    EventHandlerBuilder,
    HandleEvent,
    ReceiveMetaData,
};
use std::io::{
    Cursor,
    Error,
};
use std::ops::ControlFlow::{
    Break,
    Continue,
};

pub struct SimulatedTcpReaderEventHandler<T: TcpReadHandlerTrait> {
    read_handler: T,
}

impl<T: TcpReadHandlerTrait> SimulatedTcpReaderEventHandler<T> {
    pub fn spawn_tcp_reader(
        thread_name: String,
        single_threaded_receiver: SingleThreadedReceiver<EventOrStopThread<()>>,
        simulated_tcp_reader: SingleThreadedReceiver<Vec<u8>>,
        read_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> Result<(), Error> {
        let tcp_reader_event_handler = Self { read_handler };

        let sender =
            EventHandlerBuilder::new(&single_threaded_receiver.get_factory().clone().into())
                .spawn_thread_with_callback(thread_name, tcp_reader_event_handler, join_call_back)
                .unwrap();

        let sender_clone = sender.clone();
        simulated_tcp_reader.to_consumer(move |receive_or_disconnect| {
            match receive_or_disconnect {
                ReceiveOrDisconnected::Receive(_, buf) => {
                    return match sender_clone.send_event(buf) {
                        Ok(_) => Ok(()),
                        Err(buf) => Err(buf),
                    };
                }
                ReceiveOrDisconnected::Disconnected => {
                    let _ = sender_clone.send_stop_thread();
                    return Ok(());
                }
            };
        });

        single_threaded_receiver.to_consumer(move |receive_or_disconnect| {
            let result = match receive_or_disconnect {
                ReceiveOrDisconnected::Receive(_, EventOrStopThread::Event(())) => Ok(()),
                ReceiveOrDisconnected::Receive(_, EventOrStopThread::StopThread) => {
                    sender.send_stop_thread()
                }
                ReceiveOrDisconnected::Disconnected => sender.send_stop_thread(),
            };

            return match result {
                Ok(()) => Ok(()),
                Err(_) => Err(EventOrStopThread::StopThread),
            };
        });

        return Ok(());
    }

    pub fn new(read_handler: T) -> Self {
        return Self { read_handler };
    }

    fn read(&mut self, buf: Vec<u8>) -> EventHandleResult {
        return match rmp_serde::from_read::<Cursor<Vec<u8>>, T::ReadType>(Cursor::new(buf)) {
            Ok(read) => match self.read_handler.on_read(read) {
                Continue(()) => EventHandleResult::TryForNextEvent,
                Break(()) => EventHandleResult::StopThread,
            },
            Err(_) => EventHandleResult::StopThread,
        };
    }
}

impl<T: TcpReadHandlerTrait> HandleEvent for SimulatedTcpReaderEventHandler<T> {
    type Event = Vec<u8>;
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, buf: Self::Event) -> EventHandleResult {
        self.read(buf)
    }

    fn on_stop_self(self) -> Self::ThreadReturn {
        return ();
    }
}
