use std::sync::mpsc;

use commons::{
    factory::{
        FactoryTrait,
        RealFactory,
    },
    threading::channel::{
        RealReceiver,
        RealSender,
        RecvTimeoutError,
        SendMetaData,
        SenderTrait,
    },
    time::TimeDuration,
};
use test_utils::utils::setup_test_logging;

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

    let recieved_value1 = receiver.recv().unwrap();
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

    let recieved_value = receiver
        .recv_timeout(TimeDuration::from_millis_f64(1.0))
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
        .recv_timeout(TimeDuration::from_millis_f64(1.0))
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
        .recv_timeout(TimeDuration::from_millis_f64(-1.0))
        .unwrap_err();

    assert_eq!(RecvTimeoutError::Timeout, error);
}

#[test]
fn test_send_after_close() {
    setup_test_logging();

    let factory = RealFactory::new();
    let (sender, _)  = factory.new_channel::<i32>();
    let value = 1234;
    
    let error_value = sender.send(value).unwrap_err();

    assert_eq!(value, error_value);
}
