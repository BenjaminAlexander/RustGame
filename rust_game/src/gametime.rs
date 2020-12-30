mod timereceived;
mod timevalue;
mod timeduration;

use crate::threading::{ChannelDrivenThread, ChannelThread, Sender, ThreadBuilder, ConsumerList, Consumer};
use timer::{Timer, Guard};
use std::time::{Instant, Duration, UNIX_EPOCH, SystemTime};
use crate::threading::sender::SendError;
use log::{trace, info};
use crate::util::RollingAverage;
use std::ops::Add;
use chrono::Local;
pub use self::timereceived::TimeReceived;
use crate::gametime::timevalue::TimeValue;
pub use crate::gametime::timeduration::TimeDuration;

pub struct GameTimer {
    timer: Timer,
    duration: TimeDuration,
    start: Option<TimeValue>,
    guard: Option<Guard>,
    instantToHandle: Option<TimeValue>,
    consumer_list: ConsumerList<TimeMessage>,
    rollingAverage: RollingAverage<u64>
}

impl GameTimer {
    pub fn new(duration: TimeDuration, rollingAverageSize: usize) -> Self {

        GameTimer{
            timer: Timer::new(),
            duration,
            start: Option::None,
            guard: Option::None,
            instantToHandle: Option::None,
            consumer_list: ConsumerList::new(),
            rollingAverage: RollingAverage::new(rollingAverageSize)
        }
    }
}

impl ChannelDrivenThread<()> for GameTimer {
    fn on_none_pending(&mut self) -> Option<()> {

        if self.start.is_some() &&
            self.instantToHandle.is_some() {
            let time_message = TimeMessage{
                start: self.start.clone().unwrap(),
                duration: self.duration,
                actual_time: self.instantToHandle.unwrap(),
            };

            self.instantToHandle = None;

            self.consumer_list.accept(&time_message);
        }

        None
    }
}

impl Sender<GameTimer> {
    pub fn start(&self) -> Result<(), SendError<GameTimer>> {
        let clone = self.clone();

        self.send(|game_timer| {

            info!("Scheduling timer with duration {:?}", game_timer.duration);
            let now = TimeValue::now();

            game_timer.start = Some(now.add(game_timer.duration));
            game_timer.guard = Some(
                game_timer.timer.schedule(
                    chrono::DateTime::<Local>::from(game_timer.start.unwrap().to_system_time()),
                    Some(chrono::Duration::from_std(game_timer.duration.to_std()).unwrap()),
                    move ||clone.tick()
                )
            );
        })
    }

    pub fn add_timer_message_consumer<T>(&self, consumer: T)
        where T: Consumer<TimeMessage> {

        self.send(|game_timer|{
            game_timer.consumer_list.add_consumer(consumer);
        });

    }

    //Called from timer thread
    fn tick(&self) {
        let now = TimeValue::now();
        trace!("tick: {:?}", now);

        self.send(move |game_timer|{
            game_timer.instantToHandle = Some(now);
        }).unwrap();
    }

    pub fn on_time_message(&self, time_message: TimeReceived<TimeMessage>) {
        self.send(move |game_timer|{


        }).unwrap();
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TimeMessage {
    start: TimeValue,
    duration: TimeDuration,
    actual_time: TimeValue,
}

impl TimeMessage {
    pub fn get_sequence(&self) -> i64 {
        let duration_since_start = self.actual_time.duration_since(self.start);
        (duration_since_start / self.duration).round() as i64
    }

    pub fn get_scheduled_time(&self) -> TimeValue {
        self.start.add(self.duration * self.get_sequence())
    }

    //pub fn get_lateness(&self) -> Duration {
        //let x = self.get_scheduled_time().duration_since()
    //}
}