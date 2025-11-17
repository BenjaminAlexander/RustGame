use crate::real_time::timer_service::timer::Timer;
use crate::real_time::timer_service::{
    Schedule,
    TimerCallBack,
    TimerCreationCallBack,
    TimerId,
};
use crate::real_time::{
    EventHandleResult,
    EventHandlerBuilder,
    EventSender,
    FactoryTrait,
    HandleEvent,
    ReceiveMetaData,
    TimeSource,
};
use crate::time::TimeValue;
use log::{
    trace,
    warn,
};
use std::collections::{
    HashMap,
    VecDeque,
};
use std::io::Error;
use std::marker::PhantomData;

/// An idle [`TimerService`] that is not currently running.  Timers belonging to this service will not be called.
///
/// This struct can be used to add timers to the service before starting it, as well as starting the [`TimerService`] itself.
pub struct IdleTimerService<T: TimerCreationCallBack, U: TimerCallBack> {
    next_timer_id: usize,
    timers: VecDeque<Timer<U>>,
    unscheduled_timers: HashMap<TimerId, Timer<U>>,
    phantom: PhantomData<T>,
}

impl<T: TimerCreationCallBack, U: TimerCallBack> IdleTimerService<T, U> {
    /// Creates a new [`IdleTimerService`]
    pub fn new() -> Self {
        return Self {
            next_timer_id: 0,
            timers: VecDeque::new(),
            unscheduled_timers: HashMap::new(),
            phantom: PhantomData::default(),
        };
    }

    /// Starts the [`TimerService`] thread and begins triggering timers
    pub fn start(self, factory: &impl FactoryTrait) -> Result<TimerService<T, U>, Error> {
        let sender = EventHandlerBuilder::new_thread(
            factory,
            "TimerServiceThread".to_string(),
            TimerServiceEventHandler {
                idle_timer_service: self,
                time_source: factory.get_time_source().clone(),
            },
        )?;

        return Ok(TimerService { sender });
    }

    fn insert(&mut self, timer: Timer<U>) {
        trace!("Inserting Timer: {:?}", timer.get_id());

        if let Schedule::Never = timer.get_schedule() {
            self.unscheduled_timers.insert(*timer.get_id(), timer);
        } else {
            let index = self.timers.binary_search(&timer).unwrap_or_else(|e| e);
            self.timers.insert(index, timer);
        }
    }

    fn move_timer(&mut self, timer_id: &TimerId) -> Option<Timer<U>> {
        trace!("Moving Timer: {:?}", timer_id);
        if let Some(timer) = self.unscheduled_timers.remove(timer_id) {
            return Some(timer);
        } else {
            for i in 0..self.timers.len() {
                if let Some(timer) = self.timers.get(i) {
                    if timer.get_id() == timer_id {
                        return Some(self.timers.remove(i).unwrap());
                    }
                }
            }
        }
        return None;
    }

    fn trigger_timers(
        &mut self,
        time_source: &TimeSource,
    ) -> EventHandleResult<TimerServiceEventHandler<T, U>> {
        loop {
            let now = time_source.now();

            if let Some(timer) = self.timers.get(0) {
                if timer.should_trigger(&now) {
                    let mut timer = self.timers.pop_front().unwrap();
                    timer.trigger();

                    if timer.get_trigger_time().is_some() {
                        self.insert(timer);
                    } else {
                        self.unscheduled_timers.insert(*timer.get_id(), timer);
                    }
                } else {
                    return self.wait_for_next_trigger(time_source, now);
                }
            } else {
                return self.wait_for_next_trigger(time_source, now);
            }
        }
    }

    fn wait_for_next_trigger(
        &mut self,
        time_source: &TimeSource,
        now: TimeValue,
    ) -> EventHandleResult<TimerServiceEventHandler<T, U>> {
        if let Some(timer) = self.timers.get(0) {
            if let Some(trigger_time) = timer.get_trigger_time() {
                let duration_to_wait = trigger_time.duration_since(&now);

                if duration_to_wait.is_positive() {
                    return EventHandleResult::WaitForNextEventOrTimeout(duration_to_wait);
                } else {
                    warn!("Timers that should be triggered were left in the queue!  TimerID: {:?} Duration Until Trigger: {:?}", timer.get_id(), duration_to_wait);
                    return self.trigger_timers(time_source);
                }
            } else {
                warn!("An unscheduled timer was left in the queue!");
                let timer = self.timers.pop_front().unwrap();
                self.unscheduled_timers.insert(*timer.get_id(), timer);
                return self.trigger_timers(time_source);
            }
        } else {
            return EventHandleResult::WaitForNextEvent;
        }
    }

    /// Creates a new timer with an associated callback in the [`TimerService`]
    pub fn create_timer(&mut self, tick_call_back: U, schedule: Schedule) -> TimerId {
        let timer_id = TimerId::new(self.next_timer_id);
        self.next_timer_id = self.next_timer_id + 1;
        let timer = Timer::new(&timer_id, schedule, tick_call_back);
        self.insert(timer);
        return timer_id;
    }

    fn reschedule_timer(&mut self, timer_id: &TimerId, schedule: Schedule) {
        if let Some(mut timer) = self.move_timer(timer_id) {
            timer.set_schedule(schedule);
            self.insert(timer);
        } else {
            warn!("TimerID {:?} does not exist.", timer_id)
        }
    }

    fn cancel_timer(&mut self, timer_id: TimerId) {
        if self.move_timer(&timer_id).is_none() {
            warn!("TimerID {:?} does not exist.", timer_id)
        }
    }
}

/// A handle for a service which invokes callbacks associated with timers.
///
/// The [`TimerService`] itself runs in a another rate thread.  All callbacks
/// are invoked in this separate thread.  All timers must be added asynconously
/// via a channel, so [`TimerId`] is provided back to the caller via a callback.
///
/// Since the [`TimerService`] runs in another thread, observers may see
/// callbacks that are inconsistent with the latest [`Schedule`] for a timer
/// when a race condition occurs.
///
/// To add a timer using a sychronous call, use [`IdleTimerService`] before starting the [`TimerService`].
#[derive(Clone)]
pub struct TimerService<T: TimerCreationCallBack, U: TimerCallBack> {
    /// used to send events to the [`TimerService`] thread
    sender: EventSender<TimerServiceEvent<T, U>>,
}

impl<T: TimerCreationCallBack, U: TimerCallBack> TimerService<T, U> {
    /// Reschedules an existing timer
    pub fn reschedule_timer(&self, timer_id: TimerId, schedule: Schedule) -> Result<(), ()> {
        return simplify_result(
            self.sender
                .send_event(TimerServiceEvent::RescheduleTimer(timer_id, schedule)),
        );
    }

    /// creates a timer
    pub fn create_timer(
        &self,
        timer_creation_callback: T,
        timer_callback: U,
        schedule: Schedule,
    ) -> Result<(), ()> {
        return simplify_result(self.sender.send_event(TimerServiceEvent::CreateTimer(
            timer_creation_callback,
            timer_callback,
            schedule,
        )));
    }

    /// Removes a timer.  Once removed, the timer can never be re-used.
    pub fn remove_timer(&self, timer_id: TimerId) -> Result<(), ()> {
        return simplify_result(
            self.sender
                .send_event(TimerServiceEvent::RemoveTimer(timer_id)),
        );
    }

    //TODO: add stop function
}

//TODO: probably move to a util
fn simplify_result<T, U>(result: Result<T, U>) -> Result<T, ()> {
    return match result {
        Ok(t) => Ok(t),
        Err(_) => Err(()),
    };
}

/// The set of events that can be sent to the [`TimerService`]'s thread.
enum TimerServiceEvent<T: TimerCreationCallBack, U: TimerCallBack> {
    CreateTimer(T, U, Schedule),
    RescheduleTimer(TimerId, Schedule),
    RemoveTimer(TimerId),
}

/// An EventHandlerTrait implementation for [`TimerService`]
struct TimerServiceEventHandler<T: TimerCreationCallBack, U: TimerCallBack> {
    time_source: TimeSource,
    idle_timer_service: IdleTimerService<T, U>,
}

impl<T: TimerCreationCallBack, U: TimerCallBack> TimerServiceEventHandler<T, U> {
    fn create_timer_event_event(
        &mut self,
        creation_call_back: T,
        tick_call_back: U,
        schedule: Schedule,
    ) -> EventHandleResult<Self> {
        let timer_id = self
            .idle_timer_service
            .create_timer(tick_call_back, schedule);

        trace!(
            "Time is: {:?}\nCreated Timer: {:?}",
            self.time_source.now(),
            timer_id
        );

        creation_call_back.timer_created(&timer_id);
        return EventHandleResult::TryForNextEvent;
    }

    fn reschedule_timer_event(
        &mut self,
        timer_id: &TimerId,
        schedule: Schedule,
    ) -> EventHandleResult<Self> {
        trace!(
            "Time is: {:?}\nRescheduling Timer: {:?} to {:?}",
            self.time_source.now(),
            timer_id,
            schedule
        );

        self.idle_timer_service.reschedule_timer(timer_id, schedule);
        return EventHandleResult::TryForNextEvent;
    }

    fn cancel_timer_event(&mut self, timer_id: TimerId) -> EventHandleResult<Self> {
        trace!(
            "Time is: {:?}\nCanceling Timer: {:?}",
            self.time_source.now(),
            timer_id
        );

        self.idle_timer_service.cancel_timer(timer_id);
        return EventHandleResult::TryForNextEvent;
    }
}

impl<T: TimerCreationCallBack, U: TimerCallBack> HandleEvent for TimerServiceEventHandler<T, U> {
    type Event = TimerServiceEvent<T, U>;
    type ThreadReturn = ();

    fn on_stop(self, _: ReceiveMetaData) -> Self::ThreadReturn {
        return ();
    }

    fn on_event(&mut self, _: ReceiveMetaData, event: Self::Event) -> EventHandleResult<Self> {
        match event {
            TimerServiceEvent::CreateTimer(creation_call_back, tick_call_back, schedule) => {
                self.create_timer_event_event(creation_call_back, tick_call_back, schedule)
            }
            TimerServiceEvent::RescheduleTimer(timer_id, schedule) => {
                self.reschedule_timer_event(&timer_id, schedule)
            }
            TimerServiceEvent::RemoveTimer(timer_id) => self.cancel_timer_event(timer_id),
        }
    }

    fn on_timeout(&mut self) -> EventHandleResult<Self> {
        self.idle_timer_service.trigger_timers(&self.time_source)
    }

    fn on_channel_empty(&mut self) -> EventHandleResult<Self> {
        self.idle_timer_service.trigger_timers(&self.time_source)
    }

    fn on_channel_disconnect(&mut self) -> EventHandleResult<Self> {
        EventHandleResult::StopThread(())
    }
}
