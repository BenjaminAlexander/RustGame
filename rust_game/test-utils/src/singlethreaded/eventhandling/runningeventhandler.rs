use std::cell::RefCell;
use std::ops::ControlFlow::{Break, Continue};
use std::ops::DerefMut;
use std::rc::Rc;
use commons::threading::{AsyncJoin, ThreadBuilder};
use commons::threading::eventhandling::{ChannelEvent, ChannelEventResult, EventHandlerTrait, WaitOrTryForNextEvent};
use commons::time::TimeDuration;
use crate::singlethreaded::TimeQueue;

enum State {
    Waiting,
    //TODO: maybe combine WaitingOrTimeout and Trying
    WaitingOrTimeout(usize),
    Trying(usize),
}

pub struct RunningEventHandler<T: EventHandlerTrait, U: FnOnce(AsyncJoin<T::ThreadReturn>) + 'static> {
    event_handler: T,
    join_call_back: U,
    thread_builder: ThreadBuilder,
    state: State
}

impl <T: EventHandlerTrait, U: FnOnce(AsyncJoin<T::ThreadReturn>) + 'static> RunningEventHandler<T, U> {

    pub fn new(queue: &Rc<RefCell<TimeQueue>>, thread_builder: ThreadBuilder, event_handler: T, join_call_back: U) -> Rc<RefCell<Option<Self>>> {

        let new = Self {
            event_handler,
            join_call_back,
            thread_builder,
            state: State::Waiting
        };

        //TODO: make RC first as None then pass it in
        let rc = Rc::new(RefCell::new(Some(new)));
        rc.borrow_mut()
            .as_mut()
            .unwrap()
            .set_to_trying(&rc, queue);

        return rc;
    }

    fn set_to_waiting(&mut self, queue: &Rc<RefCell<TimeQueue>>) {

        match self.state {
            State::Waiting => {
                //NO_OP
            }
            State::WaitingOrTimeout(queue_event) => {
                TimeQueue::remove_event(&queue, queue_event);
            }
            State::Trying(queue_event) => {
                TimeQueue::remove_event(&queue, queue_event);
            }
        }

        self.state = State::Waiting;
    }

    fn set_to_trying(&mut self, self_rc: &Rc<RefCell<Option<Self>>>, queue: &Rc<RefCell<TimeQueue>>) {
        self.set_to_waiting(queue);

        let self_rc_clone = self_rc.clone();
        let queue_clone = queue.clone();
        let queue_event = TimeQueue::add_event_now(&queue, move || {
            RunningEventHandler::on_channel_event(&self_rc_clone, &queue_clone, ChannelEvent::ChannelEmpty);
        });

        self.state = State::Trying(queue_event);
    }

    fn set_to_waiting_with_timeout(&mut self, self_rc: &Rc<RefCell<Option<Self>>>, queue: &Rc<RefCell<TimeQueue>>, timeout_duration: TimeDuration) {
        self.set_to_waiting(queue);

        let self_rc_clone = self_rc.clone();
        let queue_clone = queue.clone();
        let queue_event = TimeQueue::add_event_duration_from_now(&queue, timeout_duration, move ||{
            RunningEventHandler::on_channel_event(&self_rc_clone, &queue_clone, ChannelEvent::Timeout);
        });

        self.state = State::WaitingOrTimeout(queue_event);
    }

    pub fn on_channel_event(running_handler: &Rc<RefCell<Option<Self>>>, queue: &Rc<RefCell<TimeQueue>>, event: ChannelEvent<T::Event>) {

        //TODO: rename x
        let taken = running_handler.take();

        if let Some(mut x) = taken {
            x.set_to_waiting(queue);

            match x.event_handler.on_channel_event(event) {
                Continue(WaitOrTryForNextEvent::WaitForNextEvent(event_handler)) => {
                    x.event_handler = event_handler;
                    running_handler.replace(Some(x));
                }
                Continue(WaitOrTryForNextEvent::WaitForNextEventOrTimeout(event_handler, timeout_duration)) => {
                    x.event_handler = event_handler;
                    x.set_to_waiting_with_timeout(running_handler, queue, timeout_duration);
                    running_handler.replace(Some(x));
                }
                Continue(WaitOrTryForNextEvent::TryForNextEvent(event_handler)) => {
                    x.event_handler = event_handler;
                    x.set_to_trying(running_handler, queue);
                    running_handler.replace(Some(x));
                }
                Break(result) => {
                    (x.join_call_back)(AsyncJoin::new(x.thread_builder, result));
                }
            }
        }
    }

}