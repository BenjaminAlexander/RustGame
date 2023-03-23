use std::collections::HashMap;
use std::marker::PhantomData;
use log::{trace, warn};
use crate::factory::FactoryTrait;
use crate::threading::channel::ReceiveMetaData;
use crate::threading::eventhandling::{ChannelEvent, ChannelEventResult, EventHandlerTrait, WaitOrTryForNextEvent};
use crate::threading::eventhandling::ChannelEvent::{Timeout, ChannelEmpty, ChannelDisconnected, ReceivedEvent};
use crate::time::timerservice::schedule::Schedule;
use crate::time::timerservice::timer::Timer;
use crate::time::timerservice::timercallback::TimerCallBack;
use crate::time::timerservice::timercreationcallback::TimerCreationCallBack;
use crate::time::timerservice::timerid::TimerId;
use crate::time::timerservice::timerserviceevent::TimerServiceEvent;
use crate::time::timerservice::timerserviceevent::TimerServiceEvent::{CancelTimer, CreateTimer, RescheduleTimer};
use crate::time::TimeValue;

pub struct TimeService<Factory: FactoryTrait, T: TimerCreationCallBack, U: TimerCallBack> {
    factory: Factory,
    next_timer_id: usize,

    //TODO: make this a vec deque
    timers: Vec<Timer<U>>,
    unscheduled_timers: HashMap<TimerId, Timer<U>>,
    phantom: PhantomData<T>
}

impl<Factory: FactoryTrait, T: TimerCreationCallBack, U: TimerCallBack> TimeService<Factory, T, U> {

    pub fn new(factory: Factory) -> Self {
        return Self {
            factory,
            next_timer_id: 0,
            timers: Vec::new(),
            unscheduled_timers: HashMap::new(),
            phantom: PhantomData::default()
        }
    }

    fn insert(&mut self, timer: Timer<U>) {

        trace!("Time is: {:?}\nInserting Timer: {:?}", self.factory.now(), timer.get_id());

        if timer.get_schedule().is_some() {
            let index = self.timers.binary_search(&timer).unwrap_or_else(|e| e);
            self.timers.insert(index, timer);
        } else {
            self.unscheduled_timers.insert(*timer.get_id(), timer);
        }
    }

    fn move_timer(&mut self, timer_id: &TimerId) -> Option<Timer<U>> {
        trace!("Time is: {:?}\nMoving Timer: {:?}", self.factory.now(), timer_id);
        if let Some(timer) = self.unscheduled_timers.remove(timer_id) {
            return Some(timer);
        } else {
            for i in 0..self.timers.len() {
                if let Some(timer) = self.timers.get(i) {
                    if timer.get_id() == timer_id {
                        return Some(self.timers.remove(i));
                    }
                }
            }
        }
        return None;
    }

    fn trigger_timers(mut self) -> ChannelEventResult<Self> {

        loop {
            let now = self.factory.now();

            if let Some(timer) = self.timers.get(0) {

                if timer.should_trigger(&now) {
                    let mut timer = self.timers.remove(0);
                    timer.trigger(&self.factory);

                    if timer.get_trigger_time().is_some() {
                        self.insert(timer);
                    } else {
                        self.unscheduled_timers.insert(*timer.get_id(), timer);
                    }
                } else {
                    return self.wait_for_next_trigger(now);
                }
            } else {
                return self.wait_for_next_trigger(now);
            }
        }
    }

    fn wait_for_next_trigger(mut self, now: TimeValue) -> ChannelEventResult<Self> {
        if let Some(timer) = self.timers.get(0) {
            if let Some(trigger_time) = timer.get_trigger_time() {
                let duration_to_wait = trigger_time.duration_since(&now);

                if duration_to_wait.is_positive() {
                    return ChannelEventResult::Continue(WaitOrTryForNextEvent::WaitForNextEventOrTimeout(self, duration_to_wait));
                } else {
                    warn!("Timers that should be triggered were left in the queue!  TimerID: {:?} Duration Until Trigger: {:?}", timer.get_id(), duration_to_wait);
                    return self.trigger_timers();
                }

            } else {
                warn!("An unscheduled timer was left in the queue!");
                let timer = self.timers.remove(0);
                self.unscheduled_timers.insert(*timer.get_id(), timer);
                return self.trigger_timers();
            }
        } else {
            return ChannelEventResult::Continue(WaitOrTryForNextEvent::WaitForNextEvent(self));
        }
    }

    pub fn create_timer(&mut self, tick_call_back: U, schedule: Option<Schedule>) -> TimerId {
        let timer_id = TimerId::new(self.next_timer_id);

        trace!("Time is: {:?}\nCreating Timer: {:?}", self.factory.now(), timer_id);
        self.next_timer_id = self.next_timer_id + 1;
        let timer = Timer::new(&timer_id, schedule, tick_call_back);
        self.insert(timer);
        return timer_id;
    }

    pub fn reschedule_timer(&mut self, timer_id: &TimerId, schedule: Option<Schedule>) {
        trace!("Time is: {:?}\nRescheduling Timer: {:?} to {:?}", self.factory.now(), timer_id, schedule);
        if let Some(mut timer) = self.move_timer(timer_id) {
            timer.set_schedule(schedule);
            self.insert(timer);
        } else {
            warn!("TimerID {:?} does not exist.", timer_id)
        }
    }

    pub fn cancel_timer(&mut self, timer_id: TimerId) {
        trace!("Time is: {:?}\nCanceling Timer: {:?}", self.factory.now(), timer_id);
        if self.move_timer(&timer_id).is_none() {
            warn!("TimerID {:?} does not exist.", timer_id)
        }
    }

    fn create_timer_event_event(mut self, creation_call_back: T, tick_call_back: U, schedule: Option<Schedule>) -> ChannelEventResult<Self> {
        let timer_id = self.create_timer(tick_call_back, schedule);
        creation_call_back.timer_created(&timer_id);
        return ChannelEventResult::Continue(WaitOrTryForNextEvent::TryForNextEvent(self));
    }

    fn reschedule_timer_event(mut self, timer_id: &TimerId, schedule: Option<Schedule>) -> ChannelEventResult<Self> {
        self.reschedule_timer(timer_id, schedule);
        return ChannelEventResult::Continue(WaitOrTryForNextEvent::TryForNextEvent(self));
    }

    fn cancel_timer_event(mut self, timer_id: TimerId) -> ChannelEventResult<Self> {
        self.cancel_timer(timer_id);
        return ChannelEventResult::Continue(WaitOrTryForNextEvent::TryForNextEvent(self));
    }
}

impl<Factory: FactoryTrait, T: TimerCreationCallBack, U: TimerCallBack> EventHandlerTrait for TimeService<Factory, T, U> {
    type Event = TimerServiceEvent<T, U>;
    type ThreadReturn = ();

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> ChannelEventResult<Self> {
        match channel_event {
            ReceivedEvent(_, CreateTimer(creation_call_back, tick_call_back, schedule)) => self.create_timer_event_event(creation_call_back, tick_call_back, schedule),
            ReceivedEvent(_, RescheduleTimer(timer_id, schedule)) => self.reschedule_timer_event(&timer_id, schedule),
            ReceivedEvent(_, CancelTimer(timer_id)) => self.cancel_timer_event(timer_id),
            Timeout => self.trigger_timers(),
            ChannelEmpty => self.trigger_timers(),
            ChannelDisconnected => ChannelEventResult::Break(())
        }
    }

    fn on_stop(self, _: ReceiveMetaData) -> Self::ThreadReturn {
        return ();
    }
}