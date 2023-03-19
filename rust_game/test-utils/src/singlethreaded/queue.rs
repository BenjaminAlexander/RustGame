use std::cell::RefCell;
use log::warn;
use std::collections::VecDeque;
use std::rc::Rc;
use commons::time::TimeValue;
use crate::singlethreaded::event::Event;

pub struct Queue {
    next_event_id: usize,
    queue: VecDeque<Event>,
}

impl Queue {

    pub fn new() -> Rc<RefCell<Self>> {
        return Rc::new(RefCell::new(Self {
            next_event_id: 0,
            queue: VecDeque::new()
        }));
    }

    pub fn add_event(queue: &Rc<RefCell<Self>>, time: TimeValue, function: impl FnOnce() + 'static) -> usize {
        let event_id = queue.borrow().next_event_id;
        queue.borrow_mut().next_event_id = queue.borrow().next_event_id + 1;

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
        return Self::add_event(queue, TimeValue::now(), function);
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

    pub fn run_events(queue: &Rc<RefCell<Self>>, time_value: TimeValue) {
        loop {
            if let Some(event) = queue.borrow_mut().queue.get(0) {
                if event.get_time().is_after(&time_value) {
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
}