use std::ops::ControlFlow::{Break, Continue};
use crate::gametime::{TimeValue, TimeDuration, TimeMessage, TimeReceived};
use chrono::Local;
use timer::{Guard, Timer};
use crate::util::RollingAverage;
use log::{trace, info, warn, error};
use crate::gametime::gametimer::GameTimerEvent::{InitialInformationEvent, StartTickingEvent, TickEvent, TimeMessageEvent};
use crate::gametime::gametimerobserver::GameTimerObserverTrait;
use crate::server::ServerConfig;
use crate::messaging::InitialInformation;
use crate::threading::channel::ReceiveMetaData;
use crate::threading::eventhandling::{ChannelEvent, ChannelEventResult, EventHandlerTrait, Sender};
use crate::threading::eventhandling::WaitOrTryForNextEvent::WaitForNextEvent;

const TICK_LATENESS_WARN_DURATION: TimeDuration = TimeDuration(20);
const CLIENT_ERROR_WARN_DURATION: TimeDuration = TimeDuration(20);

//TODO: should the timer be a listener that sleeps?
pub enum GameTimerEvent<Observer: GameTimerObserverTrait> {

    //TODO: pass initial information when constructing game timer
    InitialInformationEvent(InitialInformation<Observer::Game>),

    StartTickingEvent,
    //TODO: switch to sent value meta data
    TickEvent(TimeValue),
    TimeMessageEvent(TimeReceived<TimeMessage>)
}

pub struct GameTimer<Observer: GameTimerObserverTrait> {
    timer: Timer,
    server_config: Option<ServerConfig>,
    start: Option<TimeValue>,
    sender: Sender<GameTimerEvent<Observer>>,
    guard: Option<Guard>,
    rolling_average: RollingAverage<u64>,
    observer: Observer
}

impl<Observer: GameTimerObserverTrait> GameTimer<Observer> {

    pub fn new(
            rolling_average_size: usize,
            observer: Observer,
            sender: Sender<GameTimerEvent<Observer>>) -> Self {

        GameTimer{
            timer: Timer::new(),
            server_config: None,
            start: None,
            sender,
            guard: None,
            rolling_average: RollingAverage::new(rolling_average_size),
            observer
        }
    }

    pub fn get_step_duration(&self) -> Option<TimeDuration> {
        if let Some(server_config) = &self.server_config {
            return Some(server_config.get_step_duration());
        } else {
            return None;
        }
    }

    fn on_initial_information(&mut self, initial_information: InitialInformation<Observer::Game>) {
        self.server_config = Some(initial_information.move_server_config());
    }

    fn start_ticking(&mut self) {

        info!("Starting timer with duration {:?}", self.get_step_duration().unwrap());

        let now = TimeValue::now();

        self.start = Some(now.add(self.get_step_duration().unwrap()));

        let sender_clone = self.sender.clone();

        //TODO: does using this closure have bad performance?
        //Called from timer thread
        self.guard = Some(
            self.timer.schedule(
                chrono::DateTime::<Local>::from(self.start.unwrap().to_system_time()),
                Some(chrono::Duration::from_std(self.get_step_duration().unwrap().to_std()).unwrap()),
                move || {
                    if let Some(error) = sender_clone.send_event(TickEvent(TimeValue::now())).err() {
                        error!("Error while trying to send tick: {:?}", error);
                    }
                }
            )
        );
    }

    fn tick(&self, tick_time_value: TimeValue) {

        trace!("Handling Tick from {:?}", tick_time_value);

        //TODO: tick_time_value is the value from the remote thread, this value gets older and older as the event makes its way through the queue
        //TODO: How much of this can move into the other thread?
        let time_message = TimeMessage::new(
            self.start.clone().unwrap(),
            self.get_step_duration().unwrap(),
            tick_time_value);

        if time_message.get_lateness() > TICK_LATENESS_WARN_DURATION {
            warn!("High tick Lateness: {:?}", time_message.get_lateness());
        }

        self.observer.on_time_message(time_message.clone());
    }

    fn on_time_message(&mut self, time_message: TimeReceived<TimeMessage>) {
        trace!("Handling TimeMessage: {:?}", time_message);

        if let Some(step_duration) = self.get_step_duration() {

            //Calculate the start time of the remote clock in local time and add it to the rolling average
            let remote_start = time_message.get_time_received()
                .subtract(time_message.get().get_lateness())
                .subtract(step_duration * time_message.get().get_step() as i64);

            self.rolling_average.add_value(remote_start.get_millis_since_epoch() as u64);

            let average = self.rolling_average.get_average();

            if self.start.is_none() ||
                self.start.unwrap().get_millis_since_epoch() as u64 != average {

                if self.start.is_none() {
                    info!("Start client clock from signal from server clock.");
                } else {
                    let error = self.start.unwrap().get_millis_since_epoch() - average as i64;
                    if error > CLIENT_ERROR_WARN_DURATION.get_millis() {
                        warn!("High client error (millis): {:?}", error);
                    }
                }

                self.start = Some(TimeValue::from_millis(average as i64));

                let next_tick = self.start.unwrap()
                    .add(step_duration * ((TimeValue::now().duration_since(&self.start.unwrap()) / step_duration)
                        .floor() as i64 + 1));

                let sender_clone = self.sender.clone();

                //Called from timer thread
                self.guard = Some(
                    self.timer.schedule(
                        chrono::DateTime::<Local>::from(next_tick.to_system_time()),
                        Some(chrono::Duration::from_std(step_duration.to_std()).unwrap()),
                        move || {
                            if let Some(error) = sender_clone.send_event(TickEvent(TimeValue::now())).err() {
                                error!("Error while trying to send tick: {:?}", error);
                            }
                        }
                    )
                );
            }
        } else {
            warn!("TimeMessage received but ignored because this timer does not yet have a ServerConfig: {:?}", time_message);
        }
    }
}

impl<Observer: GameTimerObserverTrait> EventHandlerTrait for GameTimer<Observer> {
    type Event = GameTimerEvent<Observer>;
    type ThreadReturn = ();

    fn on_channel_event(mut self, channel_event: ChannelEvent<Self::Event>) -> ChannelEventResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(_, event) => {
                match event {
                    InitialInformationEvent(initial_information) => self.on_initial_information(initial_information),
                    StartTickingEvent => self.start_ticking(),
                    TickEvent(tick_time_value) => self.tick(tick_time_value),
                    TimeMessageEvent(time_message) => self.on_time_message(time_message)
                };

                Continue(WaitForNextEvent(self))
            }
            ChannelEvent::ChannelEmpty => Continue(WaitForNextEvent(self)),
            ChannelEvent::ChannelDisconnected => Break(())
        }
    }

    fn on_stop(self, _: ReceiveMetaData) -> Self::ThreadReturn { () }
}