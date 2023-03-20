use std::cell::RefCell;
use std::rc::Rc;
use commons::threading::{AsyncJoin, ThreadBuilder};
use commons::threading::channel::{ReceiveMetaData, SendMetaData};
use commons::threading::eventhandling::{ChannelEvent, EventHandlerTrait};
use crate::singlethreaded::eventhandling::runningeventhandler::RunningEventHandler;
use crate::singlethreaded::TimeQueue;

#[derive(Clone)]
pub struct EventHandlerHolder<T: EventHandlerTrait, U: FnOnce(AsyncJoin<T::ThreadReturn>) + 'static> {
    r: Rc<RefCell<Option<RunningEventHandler<T, U>>>>,
    queue: Rc<RefCell<TimeQueue>>,
}

impl <T: EventHandlerTrait, U: FnOnce(AsyncJoin<T::ThreadReturn>) + 'static> EventHandlerHolder<T, U> {

    pub fn new(queue: Rc<RefCell<TimeQueue>>, thread_builder: ThreadBuilder, event_handler: T, join_call_back: U) -> Self {
        return Self {
            r: RunningEventHandler::new(&queue, thread_builder, event_handler, join_call_back),
            queue
        }
    }

    pub fn on_channel_event(&self, event: ChannelEvent<T::Event>) {
        RunningEventHandler::on_channel_event(&self.r, &self.queue, event);
    }

    pub fn send(&self, event: T::Event) {
        let send_meta_data = SendMetaData::new();
        let rc_clone = self.r.clone();
        let queue_clone = self.queue.clone();

        TimeQueue::add_event_now(&self.queue, move || {
            let receive_meta_data = ReceiveMetaData::new(send_meta_data);
            RunningEventHandler::on_channel_event(&rc_clone, &queue_clone, ChannelEvent::ReceivedEvent(receive_meta_data, event));
        });
    }

}