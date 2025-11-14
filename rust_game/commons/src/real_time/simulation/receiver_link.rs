use crate::real_time::{
    ReceiveMetaData,
    SendMetaData,
    TimeSource,
};
use log::{
    error,
    warn,
};
use std::collections::VecDeque;
use std::sync::mpsc::TryRecvError;
use std::sync::{
    Arc,
    Mutex,
};

pub struct ReceiverLink<T> {
    internal: Arc<Mutex<Internal<T>>>,
    time_source: TimeSource,
}

pub enum ReceiveOrDisconnected<T> {
    Receive(ReceiveMetaData, T),
    Disconnected,
}

enum Mode<T> {
    Queue(VecDeque<(SendMetaData, T)>),
    Consumer(Box<dyn Fn(ReceiveOrDisconnected<T>) -> Result<(), T> + Send>),
    ReceiverDisconnected,
}

struct Internal<T> {
    is_sender_disconnected: bool,
    mode: Mode<T>,
}

impl<T> ReceiverLink<T> {
    pub(super) fn new(time_source: TimeSource) -> Self {
        let internal = Internal {
            is_sender_disconnected: false,
            mode: Mode::Queue(VecDeque::new()),
        };

        return Self {
            internal: Arc::new(Mutex::new(internal)),
            time_source,
        };
    }

    pub fn disconnect_receiver(&self) {
        let mut internal = self.internal.lock().unwrap();
        internal.mode = Mode::ReceiverDisconnected;
    }

    pub(super) fn disconnect_sender(&self) {
        let mut internal = self.internal.lock().unwrap();
        internal.is_sender_disconnected = true;

        if let Mode::Consumer(ref consumer) = internal.mode {
            if consumer(ReceiveOrDisconnected::Disconnected).is_err() {
                warn!("Consumer returned an error.")
            }
        }
    }

    pub(super) fn try_recv_meta_data(&self) -> Result<(ReceiveMetaData, T), TryRecvError> {
        let mut internal = self.internal.lock().unwrap();

        if let Mode::Queue(ref mut queue) = internal.mode {
            match queue.pop_front() {
                None => {
                    if internal.is_sender_disconnected {
                        return Err(TryRecvError::Disconnected);
                    } else {
                        return Err(TryRecvError::Empty);
                    }
                }
                Some((send_meta_data, t)) => {
                    let receive_meta_data = ReceiveMetaData::new(&self.time_source, send_meta_data);
                    return Ok((receive_meta_data, t));
                }
            }
        } else {
            error!("try_recv_meta_data is not allowed when not in Mode::Queue");
            panic!("try_recv_meta_data is not allowed when not in Mode::Queue");
        }
    }

    pub(super) fn to_consumer(
        &self,
        consumer: impl Fn(ReceiveOrDisconnected<T>) -> Result<(), T> + Send + 'static,
    ) {
        let mut internal = self.internal.lock().unwrap();

        if let Mode::Queue(ref mut queue) = internal.mode {
            while let Some((send_meta_data, t)) = queue.pop_front() {
                let receive_meta_data = ReceiveMetaData::new(&self.time_source, send_meta_data);

                if consumer(ReceiveOrDisconnected::Receive(receive_meta_data, t)).is_err() {
                    warn!("Consumer returned an error.")
                }
            }
        }

        if internal.is_sender_disconnected {
            if consumer(ReceiveOrDisconnected::Disconnected).is_err() {
                warn!("Consumer returned an error.")
            }
        }

        internal.mode = Mode::Consumer(Box::new(consumer));
    }

    pub(super) fn send(&self, t: T) -> Result<(), T> {
        let send_meta_data = SendMetaData::new(&self.time_source);

        let mut internal = self.internal.lock().unwrap();

        match internal.mode {
            Mode::Queue(ref mut queue) => {
                queue.push_back((send_meta_data, t));
                return Ok(());
            }
            Mode::Consumer(ref consumer) => {
                let receive_meta_data = ReceiveMetaData::new(&self.time_source, send_meta_data);
                return consumer(ReceiveOrDisconnected::Receive(receive_meta_data, t));
            }
            Mode::ReceiverDisconnected => {
                return Err(t);
            }
        }
    }
}

impl<T> Clone for ReceiverLink<T> {
    fn clone(&self) -> Self {
        return Self {
            internal: self.internal.clone(),
            time_source: self.time_source.clone(),
        };
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        mpsc::TryRecvError,
        Arc,
        Mutex,
    };

    use crate::real_time::{
        simulation::{
            receiver_link::ReceiveOrDisconnected,
            SingleThreadedFactory,
            SingleThreadedReceiver,
        },
        FactoryTrait,
    };

    #[test]
    fn test_sender() {
        let factory = SingleThreadedFactory::new();

        let (sender, mut receiver) = SingleThreadedReceiver::new(factory.clone());

        assert_eq!(TryRecvError::Empty, receiver.try_recv().unwrap_err());

        sender.send(1).unwrap();
        assert_eq!(1, receiver.try_recv().unwrap());

        sender.send(2).unwrap();

        let actual_result = Arc::new(Mutex::new(None));
        let actual_result_clone = actual_result.clone();

        let receiver_link = receiver.to_consumer(move |receive_or_disconnect| {
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
        assert_eq!(4, sender.send(4).unwrap_err());
    }

    #[test]
    fn test_drop_sender() {
        let factory = SingleThreadedFactory::new();

        let (sender, mut receiver) = factory.new_channel::<u32>();

        assert_eq!(TryRecvError::Empty, receiver.try_recv().unwrap_err());

        sender.send(1).unwrap();
        assert_eq!(1, receiver.try_recv().unwrap());

        drop(sender);
        assert_eq!(TryRecvError::Disconnected, receiver.try_recv().unwrap_err());
    }

    #[test]
    fn test_sender_error() {
        let factory = SingleThreadedFactory::new();

        let (sender, mut receiver) = SingleThreadedReceiver::new(factory.clone());

        sender.send(1).unwrap();
        assert_eq!(1, receiver.try_recv().unwrap());

        receiver.to_consumer(move |receive_or_disconnect| {
            return match receive_or_disconnect {
                ReceiveOrDisconnected::Receive(_, number) => Err(number),
                ReceiveOrDisconnected::Disconnected => Ok(()),
            };
        });

        assert_eq!(4, sender.send(4).unwrap_err());
    }
}
