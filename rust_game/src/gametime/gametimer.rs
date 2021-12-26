use std::marker::PhantomData;
use crate::gametime::{TimeValue, TimeDuration, TimeMessage, TimeReceived};
use chrono::Local;
use timer::{Guard, Timer};
use crate::threading::{ConsumerList, ChannelDrivenThread, Sender, Consumer};
use crate::util::RollingAverage;
use crate::threading::sender::SendError;
use log::{trace, info, warn};
use crate::server::ServerConfig;
use crate::messaging::InitialInformation;
use crate::interface::Game;

const TICK_LATENESS_WARN_DURATION: TimeDuration = TimeDuration(20);
const CLIENT_ERROR_WARN_DURATION: TimeDuration = TimeDuration(20);

pub struct GameTimer<GameType: Game> {
    timer: Timer,
    server_config: Option<ServerConfig>,
    start: Option<TimeValue>,
    guard: Option<Guard>,
    consumer_list: ConsumerList<TimeMessage>,
    rolling_average: RollingAverage<u64>,
    phantom: PhantomData<GameType>
}

impl<GameType: Game> GameTimer<GameType> {
    pub fn new(rolling_average_size: usize) -> Self {

        GameTimer{
            timer: Timer::new(),
            server_config: None,
            start: None,
            guard: None,
            consumer_list: ConsumerList::new(),
            rolling_average: RollingAverage::new(rolling_average_size),
            phantom: PhantomData
        }
    }

    pub fn get_step_duration(&self) -> Option<TimeDuration> {
        if let Some(server_config) = &self.server_config {
            return Some(server_config.get_step_duration());
        } else {
            return None;
        }
    }
}

impl<GameType: Game> ChannelDrivenThread<()> for GameTimer<GameType> {
    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl<GameType: Game> Sender<GameTimer<GameType>> {
    pub fn start(&self) -> Result<(), SendError<GameTimer<GameType>>> {
        let clone = self.clone();

        self.send(|game_timer| {

            info!("Starting timer with duration {:?}", game_timer.get_step_duration().unwrap());
            let now = TimeValue::now();

            game_timer.start = Some(now.add(game_timer.get_step_duration().unwrap()));
            game_timer.guard = Some(
                game_timer.timer.schedule(
                    chrono::DateTime::<Local>::from(game_timer.start.unwrap().to_system_time()),
                    Some(chrono::Duration::from_std(game_timer.get_step_duration().unwrap().to_std()).unwrap()),
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
                game_timer.get_step_duration().unwrap(),
                now);

            if time_message.get_lateness() > TICK_LATENESS_WARN_DURATION {
                warn!("High tick Lateness: {:?}", time_message.get_lateness());
            }

            game_timer.consumer_list.accept(&time_message);

        }).unwrap();
    }
}

impl<GameType: Game> Consumer<TimeReceived<TimeMessage>> for Sender<GameTimer<GameType>> {
    fn accept(&self, time_message: TimeReceived<TimeMessage>) {
        let clone = self.clone();
        self.send(move |game_timer|{
            trace!("Handling TimeMessage: {:?}", time_message);

            if let Some(step_duration) = game_timer.get_step_duration() {

                //Calculate the start time of the remote clock in local time and add it to the rolling average
                let remote_start = time_message.get_time_received()
                    .subtract(time_message.get().get_lateness())
                    .subtract(step_duration * time_message.get().get_step() as i64);

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
                        .add(step_duration * ((TimeValue::now().duration_since(game_timer.start.unwrap()) / step_duration)
                            .floor() as i64 + 1));

                    game_timer.guard = Some(
                        game_timer.timer.schedule(
                            chrono::DateTime::<Local>::from(next_tick.to_system_time()),
                            Some(chrono::Duration::from_std(step_duration.to_std()).unwrap()),
                            move ||clone.tick()
                        )
                    );
                }
            } else {
                warn!("TimeMessage received but ignored because this timer does not yet have a ServerConfig: {:?}", time_message);
            }
        }).unwrap();
    }
}

impl<GameType: Game> Consumer<InitialInformation<GameType>> for Sender<GameTimer<GameType>> {
    fn accept(&self, initial_information: InitialInformation<GameType>) {
        self.send(|game_timer|{
            game_timer.server_config = Some(initial_information.move_server_config());
        }).unwrap();
    }
}