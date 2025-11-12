use crate::{single_threaded_simulator::SingleThreadedSender, threading::{channel::{RealSender, SenderTrait}, eventhandling::EventOrStopThread}};

pub(super) enum SenderImplementation<T: Send> {
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
    pub(super) fn new(implementation: SenderImplementation<T>) -> Self {
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