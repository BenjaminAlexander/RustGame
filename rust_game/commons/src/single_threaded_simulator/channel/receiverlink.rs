use crate::single_threaded_simulator::SingleThreadedFactory;
use crate::threading::channel::{
    ReceiveMetaData,
    SendMetaData,
    TryRecvError,
};
use log::{
    error,
    warn,
};
use std::collections::VecDeque;
use std::sync::{
    Arc,
    Mutex,
};

pub struct ReceiverLink<T> {
    internal: Arc<Mutex<Internal<T>>>,
    factory: SingleThreadedFactory,
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
    pub(super) fn new(factory: SingleThreadedFactory) -> Self {
        let internal = Internal {
            is_sender_disconnected: false,
            mode: Mode::Queue(VecDeque::new()),
        };

        return Self {
            internal: Arc::new(Mutex::new(internal)),
            factory,
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
                    let receive_meta_data = ReceiveMetaData::new(&self.factory, send_meta_data);
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
                let receive_meta_data = ReceiveMetaData::new(&self.factory, send_meta_data);

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
        let send_meta_data = SendMetaData::new(&self.factory);

        let mut internal = self.internal.lock().unwrap();

        match internal.mode {
            Mode::Queue(ref mut queue) => {
                queue.push_back((send_meta_data, t));
                return Ok(());
            }
            Mode::Consumer(ref consumer) => {
                let receive_meta_data = ReceiveMetaData::new(&self.factory, send_meta_data);
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
            factory: self.factory.clone(),
        };
    }
}
