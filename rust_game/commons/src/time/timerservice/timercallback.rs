use crate::time::timerservice::timerid::TimerId;

pub trait TimerCallBack {

    fn timer_created(&mut self, timer_id: &TimerId);

    fn tick(&mut self);
}