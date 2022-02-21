use std::marker::PhantomData;
use crate::gametime::{TimeValue, TimeDuration, TimeMessage, TimeReceived};
use chrono::Local;
use timer::{Guard, Timer};
use crate::threading::{ConsumerList, ChannelDrivenThread, Sender, Consumer};
use crate::util::RollingAverage;
use crate::threading::sender::SendError;
use log::{trace, info, warn};
use crate::client::ClientCore;
use crate::gamemanager::Data;
use crate::server::{ServerConfig, ServerCore};
use crate::messaging::InitialInformation;
use crate::interface::GameTrait;

const TICK_LATENESS_WARN_DURATION: TimeDuration = TimeDuration(20);
const CLIENT_ERROR_WARN_DURATION: TimeDuration = TimeDuration(20);

pub struct GameTimer<Game: GameTrait> {
    timer: Timer,
    server_config: Option<ServerConfig>,
    start: Option<TimeValue>,
    guard: Option<Guard>,
    rolling_average: RollingAverage<u64>,
    render_receiver_sender: Sender<Data<Game>>,
    client_core_sender_option: Option<Sender<ClientCore<Game>>>,
    server_core_sender_option: Option<Sender<ServerCore<Game>>>
}

impl<Game: GameTrait> GameTimer<Game> {
    pub fn new_client_timer(rolling_average_size: usize,
            client_core_sender: Sender<ClientCore<Game>>,
            render_receiver_sender: Sender<Data<Game>>) -> Self {

        GameTimer{
            timer: Timer::new(),
            server_config: None,
            start: None,
            guard: None,
            rolling_average: RollingAverage::new(rolling_average_size),
            render_receiver_sender,
            client_core_sender_option: Some(client_core_sender),
            server_core_sender_option: None
        }
    }

    pub fn new_server_timer(rolling_average_size: usize,
            server_core_sender: Sender<ServerCore<Game>>,
            render_receiver_sender: Sender<Data<Game>>) -> Self {

        GameTimer{
            timer: Timer::new(),
            server_config: None,
            start: None,
            guard: None,
            rolling_average: RollingAverage::new(rolling_average_size),
            render_receiver_sender,
            client_core_sender_option: None,
            server_core_sender_option: Some(server_core_sender)
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

impl<Game: GameTrait> ChannelDrivenThread<()> for GameTimer<Game> {
    fn on_channel_disconnect(&mut self) -> () {
        ()
    }
}

impl<Game: GameTrait> Sender<GameTimer<Game>> {

    pub fn start(&self) -> Result<(), SendError<GameTimer<Game>>> {
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

    pub fn on_initial_information(&self, initial_information: InitialInformation<Game>) {
        self.send(|game_timer|{
            game_timer.server_config = Some(initial_information.move_server_config());
        }).unwrap();
    }

    pub fn on_time_message(&self, time_message: TimeReceived<TimeMessage>) {
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

            if let Some(client_core_sender) = game_timer.client_core_sender_option.as_ref() {
                client_core_sender.on_time_message(time_message.clone());
            }

            if let Some(server_core_sender) = game_timer.server_core_sender_option.as_ref() {
                server_core_sender.on_time_message(time_message.clone());
            }

            game_timer.render_receiver_sender.accept(time_message.clone())

        }).unwrap();
    }
}