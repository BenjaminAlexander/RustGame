use std::cell::RefCell;
use log::warn;
use std::collections::VecDeque;
use std::rc::Rc;
use commons::factory::FactoryTrait;
use commons::time::{TimeDuration, TimeValue};
use crate::singlethreaded::event::Event;
use crate::singlethreaded::singlethreadedfactory::SingleThreadedFactory;

#[derive(Clone)]
pub struct TimeQueue {
    factory: SingleThreadedFactory,
    internal: Rc<RefCell<TimeQueueInternal>>
}

struct TimeQueueInternal {
    next_event_id: usize,
    queue: VecDeque<Event>,
}

impl TimeQueue {

    pub fn new(factory: SingleThreadedFactory) -> Self {

        let internal = TimeQueueInternal {
            next_event_id: 0,
            queue: VecDeque::new()
        };

        return Self {
            factory,
            internal: Rc::new(RefCell::new(internal))
        };
    }

    pub fn get_factory(&self) -> &SingleThreadedFactory {
        return &self.factory;
    }

    pub fn add_event_at_time(&self, time: TimeValue, function: impl FnOnce() + 'static) -> usize {
        return self.internal.borrow_mut().add_event_at_time(time, function);
    }

    pub fn add_event_now(&self, function: impl FnOnce() + 'static) -> usize {
        let time = self.factory.now();
        return self.add_event_at_time(time, function);
    }

    pub fn add_event_at_duration_from_now(&self, duration: TimeDuration, function: impl FnOnce() + 'static) -> usize {
        let time = self.factory.now().add(duration);
        return self.add_event_at_time(time, function);
    }

    pub fn remove_event(&self, id: usize) {
        self.internal.borrow_mut().remove_event(id);
    }

    pub fn run_events(&self) {
        let now = self.factory.now();
        self.advance_time_until(now);
    }

    pub fn advance_time_until(&self, time_value: TimeValue) {

        loop {

            let event = self.internal.borrow_mut().pop_next_event_at_or_before(time_value);

            match event {
                Some(event) => {
                    self.factory.get_simulated_time_source().set_simulated_time(*event.get_time());
                    event.run();
                },
                None => {
                    break;
                }
            }
        }

        self.factory.get_simulated_time_source().set_simulated_time(time_value);
    }

    pub fn advance_time_for_duration(&self, time_duration: TimeDuration) {
        let time = self.factory.now().add(time_duration);
        self.advance_time_until(time);
    }
}

impl TimeQueueInternal {

    fn add_event_at_time(&mut self, time: TimeValue, function: impl FnOnce() + 'static) -> usize {
        let event_id = self.next_event_id;
        self.next_event_id = event_id + 1;

        let event = Event::new(event_id, time, function);

        let index = match self.queue.binary_search(&event) {
            Ok(index) => {
                warn!("Found a duplicate Event index");
                index
            }
            Err(index) => index
        };

        self.queue.insert(index, event);

        return index;
    }

    fn remove_event(&mut self, id: usize) {
        let mut index = 0;
        while index < self.queue.len() {

            let mut remove = false;

            if let Some(event) = self.queue.get(index) {
                if event.get_id() == id {
                    remove = true;
                }
            }

            if remove {
                self.queue.remove(index);
            } else {
                index = index + 1;
            }
        }
    }

    fn pop_next_event_at_or_before(&mut self, time_value: TimeValue) -> Option<Event> {
        if let Some(event) = self.queue.get(0) {
            if event.get_time().is_after(&time_value) {
                return None;
            }
        }

        return self.queue.pop_front();
    }
}