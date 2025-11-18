use crate::gametime::{
    GameTimerConfig,
    TimeMessage,
    TimeReceived,
};
use commons::real_time::timer_service::{
    IdleTimerService,
    Schedule,
    TimerCallBack,
    TimerCreationCallBack,
    TimerId,
    TimerService,
};
use commons::real_time::{
    Factory,
    TimeSource,
};
use commons::stats::RollingAverage;
use commons::time::{
    TimeDuration,
    TimeValue,
};
use log::{
    info,
    trace,
    warn,
};
use std::ops::{
    Add,
    Sub,
};

const TICK_LATENESS_WARN_DURATION: TimeDuration = TimeDuration::new(0, 20_000_000);
const CLIENT_ERROR_WARN_DURATION: TimeDuration = TimeDuration::new(0, 20_000_000);

pub struct GameTimer<T: TimerCallBack> {
    time_source: TimeSource,
    game_timer_config: GameTimerConfig,
    start: Option<TimeValue>,
    rolling_average: RollingAverage,
    timer_id: TimerId,
    timer_service: TimerService<GameTimerCreationCallBack, T>,
}

impl<T: TimerCallBack> GameTimer<T> {
    pub fn new(
        factory: &Factory,
        game_timer_config: GameTimerConfig,
        rolling_average_size: usize,
        call_back: T,
    ) -> Self {
        let mut idle_timer_service = IdleTimerService::new();

        let timer_id = idle_timer_service.create_timer(call_back, Schedule::Never);

        //TODO: remove unwrap
        let timer_service = idle_timer_service.start(factory).unwrap();

        return Self {
            time_source: factory.get_time_source().clone(),
            game_timer_config,
            start: None,
            rolling_average: RollingAverage::new(rolling_average_size),
            timer_id,
            timer_service,
        };
    }

    pub fn start_ticking(&mut self) -> Result<(), ()> {
        info!(
            "Starting timer with duration {:?}",
            self.game_timer_config.get_frame_duration()
        );

        let now = self.time_source.now();

        // add a frame duration to now so the first timer call back is at frame 0
        self.start = Some(now.add(&self.game_timer_config.get_frame_duration()));

        //TODO: duplicate code in on_remote_timer_message
        let schedule = Schedule::Repeating(
            self.start.unwrap(),
            *self.game_timer_config.get_frame_duration(),
        );

        let send_result = self.timer_service.reschedule_timer(self.timer_id, schedule);

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

        let step_duration = self.game_timer_config.get_frame_duration();

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
                        .time_source
                        .now()
                        .duration_since(&self.start.unwrap())
                        .as_secs_f64()
                        / step_duration.as_secs_f64())
                    .floor() as f64
                        + 1.0,
                ),
            );

            let schedule = Schedule::Repeating(
                self.start.unwrap(),
                *self.game_timer_config.get_frame_duration(),
            );

            let send_result = self.timer_service.reschedule_timer(self.timer_id, schedule);

            if send_result.is_err() {
                warn!("Failed to reschedule timer");
                return Err(());
            }
        }

        return Ok(());
    }

    pub fn create_timer_message(&self) -> TimeMessage {
        let now = self.time_source.now();

        //TODO: tick_time_value is the value from the remote thread, this value gets older and older as the event makes its way through the queue
        //TODO: How much of this can move into the other thread?
        let time_message = TimeMessage::new(
            self.start.clone().unwrap(),
            *self.game_timer_config.get_frame_duration(),
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
