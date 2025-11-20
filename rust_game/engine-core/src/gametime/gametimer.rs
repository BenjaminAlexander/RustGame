use crate::gametime::game_timer_config::{FrameIndex, StartTime};
use crate::gametime::{
    FrameDuration,
    TimeMessage,
    TimeReceived,
};
use commons::real_time::timer_service::{
    IdleTimerService, Schedule, TimerCallBack, TimerId, TimerService
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

//TODO: rename and add comments
pub struct GameTimer {
    time_source: TimeSource,
    game_timer_config: FrameDuration,
    start: Option<StartTime>,
    
    /// The FrameIndex of the frame occuring most recently in the past.
    /// This can be thought of as the current frame.
    current_frame_index: FrameIndex,
    
    rolling_average: RollingAverage,
    timer_id: TimerId,
}

impl GameTimer {
    pub fn new<T: TimerCallBack>(
        factory: &Factory,
        idle_timer_service: &mut IdleTimerService<(), T>,
        game_timer_config: FrameDuration,
        rolling_average_size: usize,
        call_back: T,
    ) -> Self {
        
        let timer_id = idle_timer_service.create_timer(call_back, Schedule::Never);

        return Self {
            time_source: factory.get_time_source().clone(),
            game_timer_config,
            start: None,
            current_frame_index: FrameIndex::zero(),
            rolling_average: RollingAverage::new(rolling_average_size),
            timer_id,
        };
    }

    pub fn start_ticking<T: TimerCallBack>(&mut self, timer_service: &TimerService<(), T>) -> Result<StartTime, ()> {
        info!(
            "Starting timer with duration {:?}",
            self.game_timer_config.get_frame_duration()
        );

        let start = StartTime::new(self.time_source.now());

        // start is now, which is the same as the time of occurance of FrameIndex 0
        self.start = Some(start);

        let next_frame_index = self.current_frame_index.next();
        let next_frame_time = start.frame_time_of_occurence(&self.game_timer_config, &next_frame_index);
        
        //TODO: duplicate code in on_remote_timer_message
        let schedule = Schedule::Repeating(
            next_frame_time,
            *self.game_timer_config.get_frame_duration(),
        );

        let send_result = timer_service.reschedule_timer(self.timer_id, schedule);

        if send_result.is_err() {
            warn!("Failed to schedule timer");
            return Err(());
        }

        //TODO: send message for FrameIndex 0

        return Ok(start);
    }

    pub fn on_remote_timer_message<T: TimerCallBack>(
        &mut self,
        timer_service: &TimerService<(), T>,
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

        if self.start.is_none() || (self.start.unwrap().time_value().as_secs_f64() - average).abs() > 1.0 {
            if self.start.is_none() {
                info!("Start client clock from signal from server clock.");
            } else {
                let error = self.start.unwrap().time_value().as_secs_f64() - average;
                if error > CLIENT_ERROR_WARN_DURATION.as_secs_f64() {
                    warn!("High client error (millis): {:?}", error);
                }
            }

            self.start = Some(StartTime::new(TimeValue::from_secs_f64(average)));

            let next_tick = self.start.unwrap().time_value().add(
                &step_duration.mul_f64(
                    (self
                        .time_source
                        .now()
                        .duration_since(&self.start.unwrap().time_value())
                        .as_secs_f64()
                        / step_duration.as_secs_f64())
                    .floor() as f64
                        + 1.0,
                ),
            );

            let schedule = Schedule::Repeating(
                *self.start.unwrap().time_value(),
                *self.game_timer_config.get_frame_duration(),
            );

            let send_result = timer_service.reschedule_timer(self.timer_id, schedule);

            if send_result.is_err() {
                warn!("Failed to reschedule timer");
                return Err(());
            }
        }

        return Ok(());
    }

    pub fn create_timer_message(&self) -> TimeMessage {
        let now = self.time_source.now();

        //TODO: work backward from now to find the most recently past FrameIndex

        //TODO: tick_time_value is the value from the remote thread, this value gets older and older as the event makes its way through the queue
        //TODO: How much of this can move into the other thread?
        let time_message = TimeMessage::new(
            *self.start.clone().unwrap().time_value(),
            *self.game_timer_config.get_frame_duration(),
            now,
        );

        if time_message.get_lateness() > TICK_LATENESS_WARN_DURATION {
            warn!("High tick Lateness: {:?}", time_message.get_lateness());
        }

        return time_message;
    }
}
