use crate::factory::FactoryTrait;
use crate::threading::channel::{
    ReceiveMetaData,
    Sender,
};
use crate::threading::eventhandling::ChannelEvent::{
    ChannelDisconnected,
    ChannelEmpty,
    ReceivedEvent,
    Timeout,
};
use crate::threading::eventhandling::{
    ChannelEvent,
    EventHandleResult,
    EventHandlerTrait,
    EventOrStopThread,
};
use crate::threading::AsyncJoin;
use crate::time::timerservice::schedule::Schedule;
use crate::time::timerservice::timer::Timer;
use crate::time::timerservice::timer_call_back::TimerCallBack;
use crate::time::timerservice::timer_creation_call_back::TimerCreationCallBack;
use crate::time::timerservice::timer_id::TimerId;
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
pub struct IdleTimerService<Factory: FactoryTrait, T: TimerCreationCallBack, U: TimerCallBack> {
    /// used to handle events when the [`TimerService`] thread starts
    event_handler: TimerServiceEventHandler<Factory, T, U>,
}

impl<Factory: FactoryTrait, T: TimerCreationCallBack, U: TimerCallBack>
    IdleTimerService<Factory, T, U>
{
    /// Creates a new [`IdleTimerService`]
    pub fn new(factory: Factory) -> Self {
        return Self {
            event_handler: TimerServiceEventHandler {
                factory,
                next_timer_id: 0,
                timers: VecDeque::new(),
                unscheduled_timers: HashMap::new(),
                phantom: PhantomData::default(),
            },
        };
    }

    /// Creates a new timer with an associated callback in the [`TimerService`]
    pub fn create_timer(&mut self, tick_call_back: U, schedule: Schedule) -> TimerId {
        return self.event_handler.create_timer(tick_call_back, schedule);
    }

    /// Starts the [`TimerService`] thread and begins triggering timers
    pub fn start(self) -> Result<TimerService<T, U>, Error> {
        let sender = self
            .event_handler
            .factory
            .new_thread_builder()
            .name("TimerServiceThread")
            .spawn_event_handler(
                self.event_handler.factory.clone(),
                self.event_handler,
                AsyncJoin::log_async_join,
            )?;

        return Ok(TimerService { sender });
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
    sender: Sender<EventOrStopThread<TimerServiceEvent<T, U>>>,
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
struct TimerServiceEventHandler<Factory: FactoryTrait, T: TimerCreationCallBack, U: TimerCallBack> {
    factory: Factory,
    next_timer_id: usize,
    timers: VecDeque<Timer<U>>,
    unscheduled_timers: HashMap<TimerId, Timer<U>>,
    phantom: PhantomData<T>,
}

impl<Factory: FactoryTrait, T: TimerCreationCallBack, U: TimerCallBack>
    TimerServiceEventHandler<Factory, T, U>
{
    fn insert(&mut self, timer: Timer<U>) {
        trace!(
            "Time is: {:?}\nInserting Timer: {:?}",
            self.factory.get_time_source().now(),
            timer.get_id()
        );

        if let Schedule::Never = timer.get_schedule() {
            self.unscheduled_timers.insert(*timer.get_id(), timer);
        } else {
            let index = self.timers.binary_search(&timer).unwrap_or_else(|e| e);
            self.timers.insert(index, timer);
        }
    }

    fn move_timer(&mut self, timer_id: &TimerId) -> Option<Timer<U>> {
        trace!(
            "Time is: {:?}\nMoving Timer: {:?}",
            self.factory.get_time_source().now(),
            timer_id
        );
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

    fn trigger_timers(mut self) -> EventHandleResult<Self> {
        loop {
            let now = self.factory.get_time_source().now();

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
                    return self.wait_for_next_trigger(now);
                }
            } else {
                return self.wait_for_next_trigger(now);
            }
        }
    }

    fn wait_for_next_trigger(mut self, now: TimeValue) -> EventHandleResult<Self> {
        if let Some(timer) = self.timers.get(0) {
            if let Some(trigger_time) = timer.get_trigger_time() {
                let duration_to_wait = trigger_time.duration_since(&now);

                if duration_to_wait.is_positive() {
                    return EventHandleResult::WaitForNextEventOrTimeout(self, duration_to_wait);
                } else {
                    warn!("Timers that should be triggered were left in the queue!  TimerID: {:?} Duration Until Trigger: {:?}", timer.get_id(), duration_to_wait);
                    return self.trigger_timers();
                }
            } else {
                warn!("An unscheduled timer was left in the queue!");
                let timer = self.timers.pop_front().unwrap();
                self.unscheduled_timers.insert(*timer.get_id(), timer);
                return self.trigger_timers();
            }
        } else {
            return EventHandleResult::WaitForNextEvent(self);
        }
    }

    fn create_timer(&mut self, tick_call_back: U, schedule: Schedule) -> TimerId {
        let timer_id = TimerId::new(self.next_timer_id);

        trace!(
            "Time is: {:?}\nCreating Timer: {:?}",
            self.factory.get_time_source().now(),
            timer_id
        );
        self.next_timer_id = self.next_timer_id + 1;
        let timer = Timer::new(&timer_id, schedule, tick_call_back);
        self.insert(timer);
        return timer_id;
    }

    pub fn reschedule_timer(&mut self, timer_id: &TimerId, schedule: Schedule) {
        trace!(
            "Time is: {:?}\nRescheduling Timer: {:?} to {:?}",
            self.factory.get_time_source().now(),
            timer_id,
            schedule
        );
        if let Some(mut timer) = self.move_timer(timer_id) {
            timer.set_schedule(schedule);
            self.insert(timer);
        } else {
            warn!("TimerID {:?} does not exist.", timer_id)
        }
    }

    pub fn cancel_timer(&mut self, timer_id: TimerId) {
        trace!(
            "Time is: {:?}\nCanceling Timer: {:?}",
            self.factory.get_time_source().now(),
            timer_id
        );
        if self.move_timer(&timer_id).is_none() {
            warn!("TimerID {:?} does not exist.", timer_id)
        }
    }

    fn create_timer_event_event(
        mut self,
        creation_call_back: T,
        tick_call_back: U,
        schedule: Schedule,
    ) -> EventHandleResult<Self> {
        let timer_id = self.create_timer(tick_call_back, schedule);
        creation_call_back.timer_created(&timer_id);
        return EventHandleResult::TryForNextEvent(self);
    }

    fn reschedule_timer_event(
        mut self,
        timer_id: &TimerId,
        schedule: Schedule,
    ) -> EventHandleResult<Self> {
        self.reschedule_timer(timer_id, schedule);
        return EventHandleResult::TryForNextEvent(self);
    }

    fn cancel_timer_event(mut self, timer_id: TimerId) -> EventHandleResult<Self> {
        self.cancel_timer(timer_id);
        return EventHandleResult::TryForNextEvent(self);
    }
}

impl<Factory: FactoryTrait, T: TimerCreationCallBack, U: TimerCallBack> EventHandlerTrait
    for TimerServiceEventHandler<Factory, T, U>
{
    type Event = TimerServiceEvent<T, U>;
    type ThreadReturn = ();

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> EventHandleResult<Self> {
        match channel_event {
            ReceivedEvent(
                _,
                TimerServiceEvent::CreateTimer(creation_call_back, tick_call_back, schedule),
            ) => self.create_timer_event_event(creation_call_back, tick_call_back, schedule),
            ReceivedEvent(_, TimerServiceEvent::RescheduleTimer(timer_id, schedule)) => {
                self.reschedule_timer_event(&timer_id, schedule)
            }
            ReceivedEvent(_, TimerServiceEvent::RemoveTimer(timer_id)) => {
                self.cancel_timer_event(timer_id)
            }
            Timeout => self.trigger_timers(),
            ChannelEmpty => self.trigger_timers(),
            ChannelDisconnected => EventHandleResult::StopThread(()),
        }
    }

    fn on_stop(self, _: ReceiveMetaData) -> Self::ThreadReturn {
        return ();
    }
}
