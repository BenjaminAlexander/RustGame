use std::cell::RefCell;
use log::warn;
use std::collections::VecDeque;
use std::rc::Rc;
use commons::time::{TimeDuration, TimeSource, TimeValue};
use crate::singlethreaded::event::Event;
use crate::time::SimulatedTimeSource;

pub struct TimeQueue {
    next_event_id: usize,
    time_source: SimulatedTimeSource,
    queue: VecDeque<Event>,
}

impl TimeQueue {

    pub fn new(time_source: SimulatedTimeSource) -> Rc<RefCell<Self>> {
        return Rc::new(RefCell::new(Self {
            next_event_id: 0,
            time_source,
            queue: VecDeque::new()
        }));
    }

    pub fn add_event_at_time(queue: &Rc<RefCell<Self>>, time: TimeValue, function: impl FnOnce() + 'static) -> usize {
        let event_id = queue.borrow().next_event_id;
        queue.borrow_mut().next_event_id = event_id + 1;

        let event = Event::new(event_id, time, function);

        let index = match queue.borrow().queue.binary_search(&event) {
            Ok(index) => {
                warn!("Found a duplicate Event index");
                index
            }
            Err(index) => index
        };

        queue.borrow_mut().queue.insert(index, event);

        return index;
    }

    pub fn add_event_now(queue: &Rc<RefCell<Self>>, function: impl FnOnce() + 'static) -> usize {
        let time = queue.borrow().time_source.now();
        return Self::add_event_at_time(queue, time, function);
    }

    pub fn add_event_duration_from_now(queue: &Rc<RefCell<Self>>, duration: TimeDuration, function: impl FnOnce() + 'static) -> usize {
        let time = queue.borrow().time_source.now().add(duration);
        return Self::add_event_at_time(queue, time, function);
    }

    pub fn remove_event(queue: &Rc<RefCell<Self>>, id: usize) {
        let mut index = 0;
        while index < queue.borrow().queue.len() {

            let mut remove = false;

            if let Some(event) = queue.borrow().queue.get(index) {
                if event.get_id() == id {
                    remove = true;
                }
            }

            if remove {
                queue.borrow_mut().queue.remove(index);
            } else {
                index = index + 1;
            }
        }
    }

    pub fn run_events(queue: &Rc<RefCell<Self>>) {
        loop {

            let now = queue.borrow().time_source.now();

            if let Some(event) = queue.borrow_mut().queue.get(0) {
                if event.get_time().is_after(&now) {
                    return;
                }
            }

            let event = queue.borrow_mut().queue.pop_front();

            match event {
                Some(event) => event.run(),
                None => { return; }
            }
        }
    }

    pub fn advance_time(queue: &Rc<RefCell<Self>>, time_value: TimeValue) {

        loop {

            let time_value_to_run_events_at;

            if let Some(event) = queue.borrow_mut().queue.get(0) {
                if event.get_time().is_after(&time_value) {
                    break;
                } else {
                    time_value_to_run_events_at = *event.get_time();
                }
            } else {
                break;
            }

            queue.borrow_mut().time_source.set_simulated_time(time_value_to_run_events_at);
            Self::run_events(queue);
        }

        queue.borrow_mut().time_source.set_simulated_time(time_value);
    }

    pub fn advance_time_for_duration(queue: &Rc<RefCell<Self>>, time_duration: TimeDuration) {
        let time = queue.borrow().time_source.now().add(time_duration);
        Self::advance_time(queue, time);
    }
}