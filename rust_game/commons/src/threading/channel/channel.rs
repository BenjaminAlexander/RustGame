use std::sync::mpsc;

use crate::{
    factory::FactoryTrait,
    single_threaded_simulator::{
        SingleThreadedFactory,
        SingleThreadedReceiver,
        SingleThreadedSender,
    },
    threading::{
        channel::{
            RealReceiver,
            RealSender,
            ReceiveMetaData,
            ReceiverTrait,
            SendMetaData,
            SenderTrait,
            TryRecvError,
        },
        eventhandling::EventOrStopThread,
    },
    time::TimeSource,
};

//TODO: cleanup
enum ChannelImplementation<T: Send> {
    Real(RealSender<T>, RealReceiver<T>),

    //TODO: conditionally compile
    Simulated(SingleThreadedSender<T>, SingleThreadedReceiver<T>),
}

pub struct Channel<Factory: FactoryTrait, T: Send + 'static> {
    sender: Sender<T>,
    receiver: Factory::Receiver<T>,
}

impl<Factory: FactoryTrait, T: Send + 'static> Channel<Factory, T> {
    pub fn new(real_sender: RealSender<T>, receiver: Factory::Receiver<T>) -> Self {
        let sender = Sender::new(SenderImplementation::Real(real_sender));
        return Self { sender, receiver };
    }

    pub fn new_simulated(
        simulated_sender: SingleThreadedSender<T>,
        receiver: Factory::Receiver<T>,
    ) -> Self {
        let sender = Sender::new(SenderImplementation::Simulated(simulated_sender));
        return Self { sender, receiver };
    }

    pub fn get_sender(&self) -> &Sender<T> {
        return &self.sender;
    }

    pub fn take(self) -> (Sender<T>, Factory::Receiver<T>) {
        return (self.sender, self.receiver);
    }
}

//TODO: cleanup
struct RealChannel<T: Send> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

enum SenderImplementation<T: Send> {
    Real(RealSender<T>),

    //TODO: conditionally compile
    Simulated(SingleThreadedSender<T>),
}

impl<T: Send> Clone for SenderImplementation<T> {
    fn clone(&self) -> Self {
        match &self {
            SenderImplementation::Real(real_sender) => {
                SenderImplementation::Real(real_sender.clone())
            }
            SenderImplementation::Simulated(simulated_sender) => {
                SenderImplementation::Simulated(simulated_sender.clone())
            }
        }
    }
}

pub struct Sender<T: Send> {
    implementation: SenderImplementation<T>,
}

impl<T: Send> Sender<T> {
    fn new(implementation: SenderImplementation<T>) -> Self {
        return Self { implementation };
    }

    pub fn send(&self, value: T) -> Result<(), T> {
        match &self.implementation {
            SenderImplementation::Real(real_sender) => real_sender.send(value),
            SenderImplementation::Simulated(simulated_sender) => simulated_sender.send(value),
        }
    }
}

impl<T: Send> Clone for Sender<T> {
    fn clone(&self) -> Self {
        return Self {
            implementation: self.implementation.clone(),
        };
    }
}

impl<T: Send> Sender<EventOrStopThread<T>> {
    pub fn send_event(&self, event: T) -> Result<(), T> {
        return match self.send(EventOrStopThread::Event(event)) {
            Ok(_) => Ok(()),
            Err(EventOrStopThread::Event(event)) => Err(event),
            _ => panic!("Unreachable"),
        };
    }

    pub fn send_stop_thread(&self) -> Result<(), ()> {
        return match self.send(EventOrStopThread::StopThread) {
            Ok(_) => Ok(()),
            Err(EventOrStopThread::StopThread) => Err(()),
            _ => panic!("Unreachable"),
        };
    }
}

enum ReceiverImplementation<T: Send> {
    Real(RealReceiver<T>),

    //TODO: conditionally compile
    Simulated(SingleThreadedReceiver<T>),
}

pub struct Receiver<T: Send> {
    implementation: ReceiverImplementation<T>,
}

impl<T: Send> Receiver<T> {
    fn new(implementation: ReceiverImplementation<T>) -> Self {
        return Self { implementation };
    }

    pub fn try_recv_meta_data(&mut self) -> Result<(ReceiveMetaData, T), TryRecvError> {
        match &mut self.implementation {
            ReceiverImplementation::Real(real_receiver) => real_receiver.try_recv_meta_data(),
            ReceiverImplementation::Simulated(simulated_receiver) => {
                simulated_receiver.try_recv_meta_data()
            }
        }
    }

    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        match &mut self.implementation {
            ReceiverImplementation::Real(real_receiver) => real_receiver.try_recv(),
            ReceiverImplementation::Simulated(simulated_receiver) => simulated_receiver.try_recv(),
        }
    }
}
