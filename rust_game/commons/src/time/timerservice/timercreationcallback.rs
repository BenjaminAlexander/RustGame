use crate::time::timerservice::timerid::TimerId;
use std::ops::Deref;

pub trait TimerCreationCallBack: Send + 'static {
    fn timer_created(&self, timer_id: &TimerId);
}

impl<T: Fn(&TimerId) + Send + 'static> TimerCreationCallBack for T {
    fn timer_created(&self, timer_id: &TimerId) {
        (&self)(timer_id);
    }
}

impl TimerCreationCallBack for Box<dyn TimerCreationCallBack> {
    fn timer_created(&self, timer_id: &TimerId) {
        (*self).deref().timer_created(timer_id);
    }
}
