use crate::gametime::game_timer_config::{FrameIndex, StartTime};
use crate::gametime::timemessage::TimeMessage;
use crate::gametime::{
    FrameDuration,
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
    //TODO:rename
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

    pub fn start_ticking<T: TimerCallBack>(&mut self, timer_service: &TimerService<(), T>) -> Result<(StartTime, FrameIndex), ()> {
        info!(
            "Starting timer with duration {:?}",
            self.game_timer_config.get_frame_duration()
        );

        let start = StartTime::new(self.time_source.now());

        // start is now, which is the same as the time of occurance of FrameIndex 0
        self.start = Some(start);

        let next_frame_index = self.current_frame_index.next();
        let next_frame_time = start.get_frame_time_of_occurence(&self.game_timer_config, &next_frame_index);
        
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

        //TODO: duplicate between this and create TimerMessage

        return Ok((start, self.current_frame_index));
    }

    //TODO: calculate ping and use that instead
    pub fn on_remote_timer_message<T: TimerCallBack>(
        &mut self,
        timer_service: &TimerService<(), T>,
        remote_frame_index: FrameIndex,
    ) -> Result<StartTime, ()> {
        trace!("Handling remote FrameIndex: {:?}", remote_frame_index);

        //Calculate the start time of the remote clock in local time and add it to the rolling average
        let remote_start = StartTime::new(self.time_source.now().sub(&self.game_timer_config.duration_from_start(&remote_frame_index)));

        self.rolling_average.add_value(remote_start.get_time_value().as_secs_f64());

        let average = self.rolling_average.get_average();

        let start = StartTime::new(TimeValue::from_secs_f64(average));

        //TODO: update logging
        if self.start.is_none() {
            info!("Start client clock from signal from server clock.");
        } else {
            let error = self.start.unwrap().get_time_value().as_secs_f64() - average;
            if error > CLIENT_ERROR_WARN_DURATION.as_secs_f64() {
                warn!("High client error (millis): {:?}", error);
            }
        }

        self.start = Some(start);

        let next_frame_index = self.current_frame_index.next();
        let next_frame_time = start.get_frame_time_of_occurence(&self.game_timer_config, &next_frame_index);
        
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

        return Ok(start);
    }

    pub fn create_timer_message(&mut self) -> Option<TimeMessage> {

        let start = match &self.start {
            Some(start) => start,
            None => return None,
        };

        let now = self.time_source.now();

        let frame_index = start.get_frame_index(&self.game_timer_config, &now);

        //TODO: maybe these logs should be trace since they can occur with timer service race conditions
        if self.current_frame_index >= frame_index {
            warn!{"Game Timer did not advance a FrameIndex since the current FrameIndex is ahead of the index calculated from the current time.  Current: {:?}, Calculated: {:?}", self.current_frame_index, frame_index};
            return None;

        } else if frame_index > self.current_frame_index.next() {
            warn!{"Game Timer advanced more than a single FrameIndex.  Current: {:?}, Advanced To: {:?}", self.current_frame_index, frame_index};
        }

        self.current_frame_index = frame_index;

        let current_frame_index_time_value = start.get_frame_time_of_occurence(&self.game_timer_config, &self.current_frame_index);

        let lateness = current_frame_index_time_value.duration_since(&now);

        if lateness > TICK_LATENESS_WARN_DURATION {
            warn!("High tick Lateness: {:?}", lateness);
        }

        //TODO: return Frame index
        return Some(TimeMessage::new(
            *start, 
            *self.game_timer_config.get_frame_duration(), 
            now));
    }
}
