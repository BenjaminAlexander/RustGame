use crate::single_threaded_simulator::channel::receiverlink::{
    ReceiveOrDisconnected,
    ReceiverLink,
};
use crate::single_threaded_simulator::channel::senderlink::SenderLink;
use crate::single_threaded_simulator::{
    SingleThreadedFactory,
    SingleThreadedSender,
};
use crate::threading::{AsyncJoinCallBackTrait, ThreadBuilder};
use crate::threading::channel::{
    ReceiveMetaData,
    ReceiverTrait,
    TryRecvError,
};
use crate::threading::eventhandling::{EventHandlerTrait, EventOrStopThread};

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

impl<T: Send> SingleThreadedReceiver<EventOrStopThread<T>> {
    pub fn spawn_thread<U: EventHandlerTrait<Event = T>>(self, thread_builder: ThreadBuilder, event_handler: U, join_call_back: impl AsyncJoinCallBackTrait<U::ThreadReturn>) -> std::io::Result<()> {


        //return Ok(());
    }
}
