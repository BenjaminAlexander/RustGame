use crate::real_time::net::udp::UdpReadHandlerTrait;
use crate::real_time::real::net::udp::{
    RealUdpSocket,
    UdpReaderEventHandler,
};
use crate::real_time::{
    real,
    EventOrStopThread,
    ReceiveMetaData,
    SendMetaData,
    TimeSource,
};
use crate::time::TimeDuration;
use std::io::Error;
use std::sync::mpsc::{
    self,
    TryRecvError,
};

pub struct RealReceiver<T: Send> {
    time_source: TimeSource,
    receiver: mpsc::Receiver<(SendMetaData, T)>, //duration_in_queue_logger: RollingStatsLogger<TimeDuration>
}

impl<T: Send> RealReceiver<T> {
    pub fn new(time_source: TimeSource, receiver: mpsc::Receiver<(SendMetaData, T)>) -> Self {
        return Self {
            time_source,
            receiver, //duration_in_queue_logger: RollingStatsLogger::new(100, 3.5, TimeDuration::from_seconds(30.0))
        };
    }

    pub fn try_recv_meta_data(&mut self) -> Result<(ReceiveMetaData, T), TryRecvError> {
        let (send_meta_data, value) = self.receiver.try_recv()?;
        return Ok((self.make_receive_meta_data(send_meta_data), value));
    }

    pub fn recv_meta_data(&mut self) -> Result<(ReceiveMetaData, T), mpsc::RecvError> {
        let (send_meta_data, value) = self.receiver.recv()?;
        return Ok((self.make_receive_meta_data(send_meta_data), value));
    }

    pub fn recv_timeout_meta_data(
        &mut self,
        duration: TimeDuration,
    ) -> Result<(ReceiveMetaData, T), mpsc::RecvTimeoutError> {
        if let Some(std_duration) = duration.to_duration() {
            let (send_meta_data, value) = self.receiver.recv_timeout(std_duration)?;
            return Ok((self.make_receive_meta_data(send_meta_data), value));
        } else {
            return Err(mpsc::RecvTimeoutError::Timeout);
        }
    }

    fn make_receive_meta_data(&mut self, send_meta_data: SendMetaData) -> ReceiveMetaData {
        let receive_meta_data = ReceiveMetaData::new(&self.time_source, send_meta_data);
        //self.duration_in_queue_logger.add_value(receive_meta_data.get_duration_in_queue());
        return receive_meta_data;
    }
}

impl RealReceiver<EventOrStopThread<()>> {
    //TODO: can these spawn methods be on a trait and called with dynamic dispatch?

    pub fn spawn_real_udp_reader<T: UdpReadHandlerTrait>(
        self,
        thread_name: String,
        udp_socket: RealUdpSocket,
        udp_read_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> Result<(), Error> {
        let event_handler = UdpReaderEventHandler::new(udp_socket, udp_read_handler);
        return real::spawn_event_handler(thread_name, self, event_handler, join_call_back);
    }
}

#[cfg(test)]
mod tests {

    use std::sync::mpsc::{
        self,
        RecvTimeoutError,
    };

    use crate::{
        logging::setup_test_logging,
        real_time::{
            real::{
                RealReceiver,
                RealSender,
            },
            FactoryTrait,
            RealFactory,
            SendMetaData,
        },
        time::TimeDuration,
    };

    #[test]
    fn test_channel() {
        setup_test_logging();

        let factory = RealFactory::new();
        let (sender, receiver) = mpsc::channel::<(SendMetaData, i32)>();
        let sender = RealSender::new(factory.get_time_source().clone(), sender);
        let mut receiver = RealReceiver::new(factory.get_time_source().clone(), receiver);

        let value1 = 1234;
        let value2 = 789;

        sender.send(value1).unwrap();

        let (_, recieved_value1) = receiver.recv_meta_data().unwrap();
        assert_eq!(value1, recieved_value1);

        sender.send(value2).unwrap();
        let (metadata2, recieved_value2) = receiver.recv_meta_data().unwrap();
        assert_eq!(value2, recieved_value2);

        assert_eq!(
            metadata2
                .get_time_received()
                .duration_since(metadata2.get_send_meta_data().get_time_sent()),
            metadata2.get_duration_in_queue()
        )
    }

    #[test]
    fn test_recv_timeout() {
        setup_test_logging();

        let factory = RealFactory::new();
        let (sender, receiver) = mpsc::channel::<(SendMetaData, i32)>();
        let sender = RealSender::new(factory.get_time_source().clone(), sender);
        let mut receiver = RealReceiver::new(factory.get_time_source().clone(), receiver);

        let value = 1234;

        sender.send(value).unwrap();

        drop(sender);

        let (_, recieved_value) = receiver
            .recv_timeout_meta_data(TimeDuration::from_millis_f64(1.0))
            .unwrap();

        assert_eq!(value, recieved_value);
    }

    #[test]
    fn test_recv_timeout_timeout() {
        setup_test_logging();

        let factory = RealFactory::new();
        let (sender, receiver) = mpsc::channel::<(SendMetaData, i32)>();
        let _sender = RealSender::new(factory.get_time_source().clone(), sender);
        let mut receiver = RealReceiver::new(factory.get_time_source().clone(), receiver);

        let recieved_value = receiver
            .recv_timeout_meta_data(TimeDuration::from_millis_f64(1.0))
            .unwrap_err();

        assert_eq!(RecvTimeoutError::Timeout, recieved_value);
    }

    #[test]
    fn test_recv_timeout_negetive_timeout() {
        setup_test_logging();

        let factory = RealFactory::new();
        let (sender, receiver) = mpsc::channel::<(SendMetaData, i32)>();
        let _sender = RealSender::new(factory.get_time_source().clone(), sender);
        let mut receiver = RealReceiver::new(factory.get_time_source().clone(), receiver);

        let error = receiver
            .recv_timeout_meta_data(TimeDuration::from_millis_f64(-1.0))
            .unwrap_err();

        assert_eq!(RecvTimeoutError::Timeout, error);
    }

    #[test]
    fn test_send_after_close() {
        setup_test_logging();

        let factory = RealFactory::new();
        let (sender, _) = factory.new_channel::<i32>();
        let value = 1234;

        let error_value = sender.send(value).unwrap_err();

        assert_eq!(value, error_value);
    }
}
