use std::cmp::Ordering;
use crate::time::timerservice::schedule::Schedule;
use crate::time::timerservice::timercallback::TimerCallBack;
use crate::time::timerservice::timerid::TimerId;
use crate::time::TimeValue;

pub struct Timer<T: TimerCallBack> {
    id: TimerId,
    schedule: Option<Schedule>,
    call_back: T
}

impl<T: TimerCallBack> Timer<T> {

    pub fn get_id(&self) -> &TimerId {
        return &self.id;
    }

    pub fn get_trigger_time(&self) -> Option<&TimeValue> {
        return match &self.schedule {
            None => None,
            Some(schedule) => Some(schedule.get_trigger_time())
        }
    }

    pub fn should_trigger(&self) -> bool {
        return match &self.schedule {
            None => false,
            Some(schedule) => schedule.should_trigger()
        }
    }

    pub fn trigger(&mut self) {
        self.call_back.tick();
        self.schedule = match self.schedule {
            None => None,
            Some(Schedule::Once(_)) => None,
            Some(Schedule::Repeating(trigger_time, duration)) => Some(Schedule::Repeating(trigger_time.add(duration), duration))
        };
    }
}

impl<T: TimerCallBack> PartialEq<Self> for Timer<T> {
    fn eq(&self, other: &Self) -> bool {
        return self.get_trigger_time().eq(&other.get_trigger_time());
    }
}

impl<T: TimerCallBack> PartialOrd<Self> for Timer<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return self.get_trigger_time().partial_cmp(&other.get_trigger_time());
    }
}

impl<T: TimerCallBack> Eq for Timer<T> {}

impl<T: TimerCallBack> Ord for Timer<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        return self.get_trigger_time().cmp(&other.get_trigger_time());
    }
}