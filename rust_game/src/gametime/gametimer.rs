use crate::gametime::{TimeValue, TimeDuration, TimeMessage, TimeReceived};
use chrono::Local;
use timer::{Guard, Timer};
use crate::threading::{ConsumerList, ChannelDrivenThread, Sender, Consumer};
use crate::util::RollingAverage;
use crate::threading::sender::SendError;
use log::{trace, info, warn};

const TICK_LATENESS_WARN_DURATION: TimeDuration = TimeDuration(20);
const CLIENT_ERROR_WARN_DURATION: TimeDuration = TimeDuration(20);

pub struct GameTimer {
    timer: Timer,
    duration: TimeDuration,
    start: Option<TimeValue>,
    guard: Option<Guard>,
    consumer_list: ConsumerList<TimeMessage>,
    rolling_average: RollingAverage<u64>
}

impl GameTimer {
    pub fn new(duration: TimeDuration, rolling_average_size: usize) -> Self {

        GameTimer{
            timer: Timer::new(),
            duration,
            start: Option::None,
            guard: Option::None,
            consumer_list: ConsumerList::new(),
            rolling_average: RollingAverage::new(rolling_average_size)
        }
    }
}

impl ChannelDrivenThread<()> for GameTimer {
    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl Sender<GameTimer> {
    pub fn start(&self) -> Result<(), SendError<GameTimer>> {
        let clone = self.clone();

        self.send(|game_timer| {

            info!("Starting timer with duration {:?}", game_timer.duration);
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
        }).unwrap();

    }

    //Called from timer thread
    fn tick(&self) {
        let now = TimeValue::now();

        self.send(move |game_timer|{
            trace!("Handling Tick from {:?}", now);

            let time_message = TimeMessage::new(
                game_timer.start.clone().unwrap(),
                game_timer.duration,
                now);

            if time_message.get_lateness() > TICK_LATENESS_WARN_DURATION {
                warn!("High tick Lateness: {:?}", time_message.get_lateness());
            }

            game_timer.consumer_list.accept(&time_message);

        }).unwrap();
    }
}

impl Consumer<TimeReceived<TimeMessage>> for Sender<GameTimer> {
    fn accept(&self, time_message: TimeReceived<TimeMessage>) {
        let clone = self.clone();
        self.send(move |game_timer|{
            trace!("Handling TimeMessage: {:?}", time_message);

            //Calculate the start time of the remote clock in local time and add it to the rolling average
            let remote_start = time_message.get_time_received()
                .subtract(time_message.get().get_lateness())
                .subtract(game_timer.duration * time_message.get().get_sequence());

            game_timer.rolling_average.add_value(remote_start.get_millis_since_epoch() as u64);

            let average = game_timer.rolling_average.get_average();

            if game_timer.start.is_none() ||
                game_timer.start.unwrap().get_millis_since_epoch() as u64 != average {

                if game_timer.start.is_none() {
                    info!("Start client clock from signal from server clock.");
                } else {
                    let error = game_timer.start.unwrap().get_millis_since_epoch() - average as i64;
                    if error > CLIENT_ERROR_WARN_DURATION.get_millis() {
                        warn!("High client error (millis): {:?}", error);
                    }
                }

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