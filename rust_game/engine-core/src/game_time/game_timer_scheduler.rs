use crate::game_time::frame_index::FrameIndex;
use crate::game_time::{
    CompletedPing,
    FrameDuration,
    StartTime,
};
use commons::real_time::timer_service::{
    IdleTimerService,
    Schedule,
    TimerCallBack,
    TimerId,
    TimerService,
};
use commons::real_time::TimeSource;
use commons::stats::RollingAverage;
use commons::time::TimeDuration;
use log::{
    info,
    warn,
};

const TICK_LATENESS_WARN_DURATION: TimeDuration = TimeDuration::new(0, 20_000_000);
const CLIENT_ERROR_WARN_DURATION: TimeDuration = TimeDuration::new(0, 20_000_000);

/// [`GameTimerScheduler`] schedules a [`TimerId`] for advancing the game's current
/// [`FrameIndex`].  This type is not an event handler so it's function depends
/// on being provided a call back that will call the appropriate methods when
/// the timer is triggered.
pub struct GameTimerScheduler {
    time_source: TimeSource,
    frame_duration: FrameDuration,
    remote_start_time: StartTime,
    start_time: StartTime,

    /// The FrameIndex of the frame occuring most recently in the past.
    /// This can be thought of as the current frame.
    current_frame_index: FrameIndex,

    rolling_average: RollingAverage,
    timer_id: TimerId,
}

impl GameTimerScheduler {
    /// Creates a [`GameTimerScheduler`] and a [`TimerId`] using the synchronous
    ///  functions on a [`IdleTimerService`] for the server.
    pub fn server_new<T: TimerCallBack>(
        time_source: TimeSource,
        idle_timer_service: &mut IdleTimerService<(), T>,
        frame_duration: FrameDuration,
        call_back: T,
    ) -> Self {
        let start_time = StartTime::new(time_source.now());
        Self::client_new(
            time_source,
            idle_timer_service,
            start_time,
            frame_duration,
            1,
            call_back,
        )
    }
    /// Creates a [`GameTimerScheduler`] and a [`TimerId`] using the synchronous
    ///  functions on a [`IdleTimerService`] for the client.
    pub fn client_new<T: TimerCallBack>(
        time_source: TimeSource,
        idle_timer_service: &mut IdleTimerService<(), T>,
        remote_start_time: StartTime,
        frame_duration: FrameDuration,
        rolling_average_size: usize,
        call_back: T,
    ) -> Self {
        let timer_id = idle_timer_service.create_timer(call_back, Schedule::Never);

        // start is now, which is the same as the time of occurance of FrameIndex 0
        let start_time = StartTime::new(time_source.now());

        return Self {
            time_source,
            frame_duration,
            remote_start_time,
            start_time,
            current_frame_index: FrameIndex::zero(),
            rolling_average: RollingAverage::new(rolling_average_size),
            timer_id,
        };
    }

    /// Starts the server's game timer
    pub fn start_server_timer<T: TimerCallBack>(
        &mut self,
        timer_service: &TimerService<(), T>,
    ) -> Result<(StartTime, FrameIndex), ()> {
        info!(
            "Starting timer with duration {:?}",
            self.frame_duration.get_frame_duration()
        );

        let zero_time_ping = CompletedPing::zero_time_ping();
        self.adjust_client_timer(timer_service, zero_time_ping)?;

        return Ok((self.start_time, self.current_frame_index));
    }

    /// Adjust the client's server-client clock offset and reschedules
    pub fn adjust_client_timer<T: TimerCallBack>(
        &mut self,
        timer_service: &TimerService<(), T>,
        completed_ping: CompletedPing,
    ) -> Result<StartTime, ()> {
        //Calculate the start time of the remote clock in local time and add it to the rolling average
        let offset = completed_ping.get_remote_to_local_clock_offset();
        self.rolling_average.add_value(offset);
        let average_offset = self.rolling_average.get_average();

        let start = CompletedPing::get_local_start_time(average_offset, &self.remote_start_time);

        let error = (self.start_time.get_time_value().as_secs_f64()
            - start.get_time_value().as_secs_f64())
        .abs();
        if error > CLIENT_ERROR_WARN_DURATION.as_secs_f64() {
            warn!("High client error (sec f64): {:?}", error);
        }

        self.start_time = start;

        let next_frame_index = self.current_frame_index.next();
        let next_frame_time =
            start.get_frame_time_of_occurence(&self.frame_duration, &next_frame_index);

        let schedule =
            Schedule::Repeating(next_frame_time, *self.frame_duration.get_frame_duration());

        let send_result = timer_service.reschedule_timer(self.timer_id, schedule);

        if send_result.is_err() {
            warn!("Failed to schedule timer");
            return Err(());
        }

        return Ok(start);
    }

    /// Tries to advance the current [`FrameIndex`] to the next frame.  This
    /// function will never return the same [`FrameIndex`] twice.  It will also
    /// never decrease the [`FrameIndex`].  This function may advance the [`FrameIndex`]
    /// more than one increment if multiple [`FrameDuration`]s have elapsed since
    /// the last time it was called.
    pub fn try_advance_frame_index(&mut self) -> Option<FrameIndex> {
        let now = self.time_source.now();

        let mut frame_index = self.start_time.get_frame_index(&self.frame_duration, &now);

        if self.current_frame_index == frame_index {
            // It seems common for the clock to be slightly early on clients, probably
            // due to race conditions with rescheduling the timer and adjusting the
            // client's start time.  It may be good enough to advance the frame if the
            // calculated frame index is the same as the current one.
            frame_index = frame_index.next();
        } else if self.current_frame_index > frame_index {
            let next_frame_time = self.start_time.get_frame_time_of_occurence(
                &self.frame_duration,
                &self.current_frame_index.next(),
            );
            let time_until_next_frame = next_frame_time.duration_since(&now);
            warn!("Game Timer did not advance a FrameIndex since the current FrameIndex is ahead of the index calculated from the current time.  Current: {:?}, Calculated: {:?}, Time until next frame: {:?}", self.current_frame_index, frame_index, time_until_next_frame);
            return None;
        } else if frame_index > self.current_frame_index.next() {
            warn!("Game Timer advanced more than a single FrameIndex.  Current: {:?}, Advanced To: {:?}", self.current_frame_index, frame_index);
        }

        self.current_frame_index = frame_index;

        let current_frame_index_time_value = self
            .start_time
            .get_frame_time_of_occurence(&self.frame_duration, &self.current_frame_index);
        let lateness = now.duration_since(&current_frame_index_time_value);

        if lateness > TICK_LATENESS_WARN_DURATION {
            warn!("High tick Lateness: {:?}", lateness);
        }

        return Some(self.current_frame_index);
    }

    pub fn get_current_frame_index(&self) -> FrameIndex {
        self.current_frame_index
    }

    pub fn get_start_time(&self) -> StartTime {
        self.start_time
    }
}
