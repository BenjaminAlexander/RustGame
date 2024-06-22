use crate::singlethreaded::channel::receiverlink::{ReceiveOrDisconnected, ReceiverLink};
use crate::singlethreaded::channel::senderlink::SenderLink;
use crate::singlethreaded::{SingleThreadedFactory, SingleThreadedSender};
use commons::threading::channel::{ReceiveMetaData, ReceiverTrait, TryRecvError};

pub struct SingleThreadedReceiver<T: Send> {
    link: ReceiverLink<T>,
}

impl<T: Send> ReceiverTrait<T> for SingleThreadedReceiver<T> {
    fn try_recv_meta_data(&mut self) -> Result<(ReceiveMetaData, T), TryRecvError> {
        return self.link.try_recv_meta_data();
    }
}

impl<T: Send> SingleThreadedReceiver<T> {
    pub fn new(factory: SingleThreadedFactory) -> (SingleThreadedSender<T>, Self) {
        let receiver_link = ReceiverLink::new(factory);
        let sender_link = SenderLink::new(receiver_link.clone());
        let sender = SingleThreadedSender::new(sender_link);
        let receiver = Self {
            link: receiver_link,
        };

        return (sender, receiver);
    }

    pub fn to_consumer(
        self,
        consumer: impl Fn(ReceiveOrDisconnected<T>) -> Result<(), T> + Send + 'static,
    ) -> ReceiverLink<T> {
        self.link.to_consumer(consumer);
        return self.link;
    }
}
