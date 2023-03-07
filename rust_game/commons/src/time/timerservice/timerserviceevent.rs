use crate::time::timerservice::schedule::Schedule;
use crate::time::timerservice::timercallback::TimerCallBack;
use crate::time::timerservice::timerid::TimerId;

enum TimerServiceEvent<T: TimerCallBack> {
    CreateTimer(T, Schedule),
    RescheduleTimer(TimerId, Schedule),
    CancelTimer(TimerId)
}