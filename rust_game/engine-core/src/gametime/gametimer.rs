use std::ops::{
    Add,
    Sub,
};

use crate::gametime::{
    TimeMessage,
    TimeReceived,
};
use crate::server::ServerConfig;
use commons::factory::FactoryTrait;
use commons::stats::RollingAverage;
use commons::threading::eventhandling::{
    EventHandlerSender,
    EventSenderTrait,
};
use commons::threading::AsyncJoin;
use commons::time::timerservice::{
    Schedule,
    TimeService,
    TimerCallBack,
    TimerCreationCallBack,
    TimerId,
    TimerServiceEvent,
};
use commons::time::{
    TimeDuration,
    TimeValue,
};
use log::{
    info,
    trace,
    warn,
};
use timer::Timer;

const TICK_LATENESS_WARN_DURATION: TimeDuration = TimeDuration::new(0, 20_000_000);
const CLIENT_ERROR_WARN_DURATION: TimeDuration = TimeDuration::new(0, 20_000_000);

pub struct GameTimer<Factory: FactoryTrait, T: TimerCallBack> {
    factory: Factory,
    //TODO: remove timer
    timer: Timer,
    server_config: ServerConfig,
    start: Option<TimeValue>,
    rolling_average: RollingAverage,
    new_timer_id: TimerId,
    new_timer_sender: EventHandlerSender<Factory, TimerServiceEvent<GameTimerCreationCallBack, T>>,
}

impl<Factory: FactoryTrait, T: TimerCallBack> GameTimer<Factory, T> {
    pub fn new(
        factory: Factory,
        server_config: ServerConfig,
        rolling_average_size: usize,
        call_back: T,
    ) -> Self {
        let mut timer_service =
            TimeService::<Factory, GameTimerCreationCallBack, T>::new(factory.clone());

        let timer_id = timer_service.create_timer(call_back, None);

        let new_timer_sender = factory
            .new_thread_builder()
            .name("NewTimerThread")
            .spawn_event_handler(timer_service, AsyncJoin::log_async_join)
            .unwrap();

        return Self {
            factory,
            timer: Timer::new(),
            server_config,
            start: None,
            rolling_average: RollingAverage::new(rolling_average_size),
            new_timer_id: timer_id,
            new_timer_sender,
        };
    }

    pub fn start_ticking(&mut self) -> Result<(), ()> {
        info!(
            "Starting timer with duration {:?}",
            self.server_config.get_step_duration()
        );

        let now = self.factory.now();

        self.start = Some(now.add(&self.server_config.get_step_duration()));

        let schedule =
            Schedule::Repeating(self.start.unwrap(), self.server_config.get_step_duration());

        let send_result = self
            .new_timer_sender
            .send_event(TimerServiceEvent::RescheduleTimer(
                self.new_timer_id,
                Some(schedule),
            ));

        if send_result.is_err() {
            warn!("Failed to schedule timer");
            return Err(());
        }

        return Ok(());
    }

    pub fn on_remote_timer_message(
        &mut self,
        time_message: TimeReceived<TimeMessage>,
    ) -> Result<(), ()> {
        trace!("Handling TimeMessage: {:?}", time_message);

        let step_duration = self.server_config.get_step_duration();

        //Calculate the start time of the remote clock in local time and add it to the rolling average
        let remote_start = time_message
            .get_time_received()
            .sub(&time_message.get().get_lateness())
            .sub(&step_duration.mul_f64(time_message.get().get_step() as f64));

        self.rolling_average.add_value(remote_start.as_secs_f64());

        let average = self.rolling_average.get_average();

        if self.start.is_none() || (self.start.unwrap().as_secs_f64() - average).abs() > 1.0 {
            if self.start.is_none() {
                info!("Start client clock from signal from server clock.");
            } else {
                let error = self.start.unwrap().as_secs_f64() - average;
                if error > CLIENT_ERROR_WARN_DURATION.as_secs_f64() {
                    warn!("High client error (millis): {:?}", error);
                }
            }

            self.start = Some(TimeValue::from_secs_f64(average));

            let next_tick = self.start.unwrap().add(
                &step_duration.mul_f64(
                    (self
                        .factory
                        .now()
                        .duration_since(&self.start.unwrap())
                        .as_secs_f64()
                        / step_duration.as_secs_f64())
                    .floor() as f64
                        + 1.0,
                ),
            );

            let schedule =
                Schedule::Repeating(self.start.unwrap(), self.server_config.get_step_duration());

            let send_result = self
                .new_timer_sender
                .send_event(TimerServiceEvent::RescheduleTimer(
                    self.new_timer_id,
                    Some(schedule),
                ));

            if send_result.is_err() {
                warn!("Failed to reschedule timer");
                return Err(());
            }
        }

        return Ok(());
    }

    pub fn create_timer_message(&self) -> TimeMessage {
        let now = self.factory.now();

        //TODO: tick_time_value is the value from the remote thread, this value gets older and older as the event makes its way through the queue
        //TODO: How much of this can move into the other thread?
        let time_message = TimeMessage::new(
            self.start.clone().unwrap(),
            self.server_config.get_step_duration(),
            now,
        );

        if time_message.get_lateness() > TICK_LATENESS_WARN_DURATION {
            warn!("High tick Lateness: {:?}", time_message.get_lateness());
        }

        return time_message;
    }
}

struct GameTimerCreationCallBack {}

impl TimerCreationCallBack for GameTimerCreationCallBack {
    fn timer_created(&self, timer_id: &TimerId) {
        warn!("Timer Created: {:?}", timer_id);
    }
}
