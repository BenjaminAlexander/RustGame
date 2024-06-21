use crate::time::timerservice::schedule::Schedule;
use crate::time::timerservice::timercallback::TimerCallBack;
use crate::time::timerservice::timercreationcallback::TimerCreationCallBack;
use crate::time::timerservice::timerid::TimerId;

pub enum TimerServiceEvent<T: TimerCreationCallBack, U: TimerCallBack> {
    CreateTimer(T, U, Option<Schedule>),
    RescheduleTimer(TimerId, Option<Schedule>),
    CancelTimer(TimerId),
}
