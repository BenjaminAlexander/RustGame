use std::sync::mpsc::TryRecvError;

use crate::real_time::{
    real::RealReceiver,
    simulation::SingleThreadedReceiver,
    ReceiveMetaData,
};

pub(super) enum ReceiverImplementation<T: Send> {
    Real(RealReceiver<T>),

    //TODO: conditionally compile
    Simulated(SingleThreadedReceiver<T>),
}

pub struct Receiver<T: Send> {
    implementation: ReceiverImplementation<T>,
}

impl<T: Send> Receiver<T> {
    pub(super) fn new(implementation: ReceiverImplementation<T>) -> Self {
        return Self { implementation };
    }

    pub(super) fn take_implementation(self) -> ReceiverImplementation<T> {
        return self.implementation;
    }
}

impl<T: Send> Receiver<T> {
    pub fn try_recv_meta_data(&mut self) -> Result<(ReceiveMetaData, T), TryRecvError> {
        match &mut self.implementation {
            ReceiverImplementation::Real(real_receiver) => real_receiver.try_recv_meta_data(),
            ReceiverImplementation::Simulated(simulated_receiver) => {
                simulated_receiver.try_recv_meta_data()
            }
        }
    }

    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        let (_, value) = self.try_recv_meta_data()?;
        return Ok(value);
    }
}

