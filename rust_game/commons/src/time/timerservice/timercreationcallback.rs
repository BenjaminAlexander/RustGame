use crate::time::timerservice::timerid::TimerId;

pub trait TimerCreationCallBack: Send + 'static {

    fn timer_created(self, timer_id: &TimerId);

}