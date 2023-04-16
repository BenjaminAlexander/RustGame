use std::sync::{Arc, mpsc, Mutex};
use commons::factory::FactoryTrait;
use commons::threading::channel::{ReceiverTrait, SenderTrait, TryRecvError};
use test_utils::singlethreaded::{ReceiveOrDisconnected, SingleThreadedFactory};

#[test]
fn test_sender() {

    let factory = SingleThreadedFactory::new();

    let (sender, mut receiver) = factory.new_channel::<u32>().take();

    assert_eq!(TryRecvError::Empty, receiver.try_recv().unwrap_err());

    sender.send(1).unwrap();
    assert_eq!(1, receiver.try_recv().unwrap());

    sender.send(2).unwrap();

    let actual_result = Arc::new(Mutex::new(None));
    let actual_result_clone = actual_result.clone();

    let receiver_link = receiver.to_consumer(move |receive_or_disconnect|{
        match receive_or_disconnect {
            ReceiveOrDisconnected::Receive(_, number) => {
                *actual_result_clone.lock().unwrap() = Some(number);
            }
            ReceiveOrDisconnected::Disconnected => {}
        }
       return Ok(());
    });

    assert_eq!(2, *actual_result.lock().unwrap().as_ref().unwrap());

    sender.send(3).unwrap();
    assert_eq!(3, *actual_result.lock().unwrap().as_ref().unwrap());

    receiver_link.disconnect_receiver();
    assert_eq!(4, sender.send(4).unwrap_err().0.1);
}

#[test]
fn test_drop_sender() {
    let factory = SingleThreadedFactory::new();

    let (sender, mut receiver) = factory.new_channel::<u32>().take();

    assert_eq!(TryRecvError::Empty, receiver.try_recv().unwrap_err());

    sender.send(1).unwrap();
    assert_eq!(1, receiver.try_recv().unwrap());

    drop(sender);
    assert_eq!(TryRecvError::Disconnected, receiver.try_recv().unwrap_err());
}

#[test]
fn test_sender_error() {
    let factory = SingleThreadedFactory::new();

    let (sender, mut receiver) = factory.new_channel::<u32>().take();

    sender.send(1).unwrap();
    assert_eq!(1, receiver.try_recv().unwrap());

    receiver.to_consumer(move |receive_or_disconnect|{
        return match receive_or_disconnect {
            ReceiveOrDisconnected::Receive(receive_meta_data, number) => Err(mpsc::SendError((receive_meta_data.get_send_meta_data().clone(), number))),
            ReceiveOrDisconnected::Disconnected => Ok(())
        };
    });

    assert_eq!(4, sender.send(4).unwrap_err().0.1);
}