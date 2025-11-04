use std::sync::mpsc;

use crate::{single_threaded_simulator::{SingleThreadedFactory, SingleThreadedReceiver, SingleThreadedSender}, threading::channel::{RealReceiver, RealSender, ReceiveMetaData, ReceiverTrait, SendMetaData, SenderTrait, TryRecvError}, time::TimeSource};

//TODO: cleanup
enum ChannelImplementation<T: Send> {
    Real(RealSender<T>, RealReceiver<T>),

    //TODO: conditionally compile
    Simulated(SingleThreadedSender<T>, SingleThreadedReceiver<T>)
}

pub struct Channel<T: Send + 'static> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

impl<T: Send + 'static> Channel<T> {

    pub fn new_real_channel(time_source: TimeSource) -> (RealSender<T>, RealReceiver<T>) {
        let (sender, receiver) = mpsc::channel::<(SendMetaData, T)>();
        let sender = RealSender::new(time_source.clone(), sender);
        let receiver = RealReceiver::new(time_source, receiver);
        return (sender, receiver);
    }

    //TODO: clean this
    pub fn new_simulated_channel(factory: SingleThreadedFactory) -> (SingleThreadedSender<T>, SingleThreadedReceiver<T>) {
        return SingleThreadedReceiver::new(factory);
    }

    pub fn new(time_source: TimeSource) -> Self {
        let (sender, receiver) = mpsc::channel::<(SendMetaData, T)>();
        let sender = RealSender::new(time_source.clone(), sender);
        let sender = Sender::new(SenderImplementation::Real(sender));
        let receiver = RealReceiver::new(time_source, receiver);
        let receiver = Receiver::new(ReceiverImplementation::Real(receiver));
        return Self { sender, receiver };
    }

    pub fn new_simulated(factory: SingleThreadedFactory) -> Self {
        let (sender, receiver) = SingleThreadedReceiver::new(factory);
        let sender = Sender::new(SenderImplementation::Simulated(sender));
        let receiver = Receiver::new(ReceiverImplementation::Simulated(receiver));
        return Self { sender, receiver };
    }

    pub fn get_sender(&self) -> &Sender<T> {
        return &self.sender;
    }

    pub fn take(self) -> (Sender<T>, Receiver<T>) {
        return (self.sender, self.receiver);
    }
}

//TODO: cleanup
struct RealChannel<T: Send> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

#[derive(Clone)]
enum SenderImplementation<T: Send> {
    Real(RealSender<T>),

    //TODO: conditionally compile
    Simulated(SingleThreadedSender<T>)
}

impl<T: Send> SenderImplementation<T> {
    fn clone(&self) -> Self {
        match &self {
            SenderImplementation::Real(real_sender) => SenderImplementation::Real(real_sender.clone()),
            SenderImplementation::Simulated(simulated_sender) => SenderImplementation::Simulated(simulated_sender.clone())
        }
    }
}

#[derive(Clone)]
pub struct Sender<T: Send> {
    implementation: SenderImplementation<T>
}

impl<T: Send> Sender<T> {

    fn new(implementation: SenderImplementation<T>) -> Self {
        return Self { implementation }
    }

    pub fn send(&self, value: T) -> Result<(), T> {
        match &self.implementation {
            SenderImplementation::Real(real_sender) => real_sender.send(value),
            SenderImplementation::Simulated(simulated_sender) => simulated_sender.send(value),
        }
    }

    pub fn clone(&self) -> Self {
        return Self {
            implementation: self.implementation.clone()
        };
    }
}

enum ReceiverImplementation<T: Send> {
    Real(RealReceiver<T>),

    //TODO: conditionally compile
    Simulated(SingleThreadedReceiver<T>)
}

pub struct Receiver<T: Send> {
    implementation: ReceiverImplementation<T>
}

impl<T: Send> Receiver<T> {

    fn new(implementation: ReceiverImplementation<T>) -> Self {
        return Self { implementation }
    }

    pub fn try_recv_meta_data(&mut self) -> Result<(ReceiveMetaData, T), TryRecvError> {
        match &mut self.implementation {
            ReceiverImplementation::Real(real_receiver) => real_receiver.try_recv_meta_data(),
            ReceiverImplementation::Simulated(simulated_receiver) => simulated_receiver.try_recv_meta_data(),
        }
    }

    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        match &mut self.implementation {
            ReceiverImplementation::Real(real_receiver) => real_receiver.try_recv(),
            ReceiverImplementation::Simulated(simulated_receiver) => simulated_receiver.try_recv(),
        }
    }
}