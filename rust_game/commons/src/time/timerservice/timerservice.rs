
use std::collections::HashMap;
use crate::time::timerservice::timer::Timer;
use crate::time::timerservice::timercallback::TimerCallBack;
use crate::time::timerservice::timerid::TimerId;

pub struct TimeService<'a, T: TimerCallBack> {
    timers: HashMap<TimerId, Timer<T>>,
    order: Vec<&'a mut Timer<T>>
}

impl<'a, T: TimerCallBack> TimeService<'a, T> {

    fn insert<'b: 'a>(&'b mut self, timer: Timer<T>) {
        let id = *timer.get_id();
        self.timers.insert(id, timer);
        let timer_ref = self.timers.get_mut(&id).unwrap();

        //self.insert_into_queue(timer_ref);

        let index = self.order.binary_search(&timer_ref).unwrap_or_else(|e| e);
        self.order.insert(index, timer_ref);
    }

    fn insert_into_queue(&mut self, timer: &'a mut Timer<T>) {
        let index = self.order.binary_search(&timer).unwrap_or_else(|e| e);
        self.order.insert(index, timer);
    }

    fn do_next(&mut self) {

        if let Some(timer) = self.order.get(0) {
            if timer.should_trigger() {
                let timer = self.order.remove(0);
                timer.trigger();

                if timer.get_trigger_time().is_some() {
                    self.insert_into_queue(timer);
                }
            }
        }

    }
}
