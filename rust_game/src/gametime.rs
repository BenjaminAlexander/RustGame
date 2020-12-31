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
pub use crate::gametime::timevalue::TimeValue;
pub use crate::gametime::timeduration::TimeDuration;

pub struct GameTimer {
    timer: Timer,
    duration: TimeDuration,
    start: Option<TimeValue>,
    guard: Option<Guard>,
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
            consumer_list: ConsumerList::new(),
            rollingAverage: RollingAverage::new(rollingAverageSize)
        }
    }
}

impl ChannelDrivenThread<()> for GameTimer {

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

        self.send(move |game_timer|{
            let time_message = TimeMessage{
                start: game_timer.start.clone().unwrap(),
                duration: game_timer.duration,
                actual_time: now,
            };

            game_timer.consumer_list.accept(&time_message);

        }).unwrap();
    }

    pub fn on_time_message(&self, time_message: TimeReceived<TimeMessage>) {
        let clone = self.clone();
        self.send(move |game_timer|{

            //Calculate the start time of the remote clock in local time and add it to the rolling average
            let remote_start = time_message.get_time_received()
                .subtract(time_message.get().get_lateness())
                .subtract(game_timer.duration * time_message.get().get_sequence());

            game_timer.rollingAverage.add_value(remote_start.get_millis_since_epoch() as u64);

            let average = game_timer.rollingAverage.get_average();

            if game_timer.start.is_none() ||
                game_timer.start.unwrap().get_millis_since_epoch() as u64 != average {

                game_timer.start = Some(TimeValue::from_millis(average as i64));

                let next_tick = game_timer.start.unwrap()
                    .add(game_timer.duration * ((TimeValue::now().duration_since(game_timer.start.unwrap()) / game_timer.duration)
                    .floor() as i64 + 1));

                game_timer.guard = Some(
                    game_timer.timer.schedule(
                        chrono::DateTime::<Local>::from(next_tick.to_system_time()),
                        Some(chrono::Duration::from_std(game_timer.duration.to_std()).unwrap()),
                        move ||clone.tick()
                    )
                );
            }
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

    pub fn get_lateness(&self) -> TimeDuration {
        self.get_scheduled_time().duration_since(self.actual_time)
    }
}