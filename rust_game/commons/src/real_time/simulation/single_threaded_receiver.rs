use crate::real_time::simulation::receiver_link::{
    ReceiveOrDisconnected,
    ReceiverLink,
};
use crate::real_time::simulation::sender_link::SenderLink;
use crate::real_time::simulation::single_threaded_sender::SingleThreadedSender;
use crate::real_time::simulation::SingleThreadedFactory;
use crate::real_time::{
    FactoryTrait,
    ReceiveMetaData,
};
use std::sync::mpsc::TryRecvError;

pub struct SingleThreadedReceiver<T: Send> {
    factory: SingleThreadedFactory,
    link: ReceiverLink<T>,
}

impl<T: Send> SingleThreadedReceiver<T> {
    pub fn new(factory: SingleThreadedFactory) -> (SingleThreadedSender<T>, Self) {
        let receiver_link = ReceiverLink::new(factory.get_time_source().clone());
        let sender_link = SenderLink::new(receiver_link.clone());
        let sender = SingleThreadedSender::new(sender_link);
        let receiver = Self {
            factory,
            link: receiver_link,
        };

        return (sender, receiver);
    }

    pub fn get_factory(&self) -> &SingleThreadedFactory {
        return &self.factory;
    }

    pub fn try_recv_meta_data(&mut self) -> Result<(ReceiveMetaData, T), TryRecvError> {
        return self.link.try_recv_meta_data();
    }

    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        let (_, value) = self.try_recv_meta_data()?;
        return Ok(value);
    }

    pub fn to_consumer(
        self,
        consumer: impl Fn(ReceiveOrDisconnected<T>) -> Result<(), T> + Send + 'static,
    ) -> ReceiverLink<T> {
        self.link.to_consumer(consumer);
        return self.link;
    }
}
