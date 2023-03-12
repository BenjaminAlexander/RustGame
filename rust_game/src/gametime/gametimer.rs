use std::ops::ControlFlow::{Break, Continue};
use commons::stats::RollingAverage;
use commons::time::{TimeValue, TimeDuration};
use crate::gametime::{TimeMessage, TimeReceived};
use chrono::Local;
use timer::{Guard, Timer};
use log::{trace, info, warn, error};
use crate::gametime::gametimer::GameTimerEvent::{StartTickingEvent, TickEvent, TimeMessageEvent};
use crate::gametime::gametimerobserver::GameTimerObserverTrait;
use crate::server::ServerConfig;
use commons::threading::channel::ReceiveMetaData;
use commons::threading::eventhandling::{ChannelEvent, ChannelEventResult, EventHandlerTrait, Sender};
use commons::threading::eventhandling::WaitOrTryForNextEvent::{TryForNextEvent, WaitForNextEvent};
use commons::threading::{AsyncJoin, ThreadBuilder};
use commons::time::timerservice::{Schedule, TimerCallBack, TimerCreationCallBack, TimerId, TimerServiceEvent, TimeService};

const TICK_LATENESS_WARN_DURATION: TimeDuration = TimeDuration::from_seconds(0.02);
const CLIENT_ERROR_WARN_DURATION: TimeDuration = TimeDuration::from_seconds(0.02);

//TODO: should the timer be a listener that sleeps?
pub enum GameTimerEvent {
    StartTickingEvent,
    //TODO: switch to sent value meta data
    TickEvent(TimeValue),
    TimeMessageEvent(TimeReceived<TimeMessage>)
}

pub struct GameTimer<Observer: GameTimerObserverTrait> {
    timer: Timer,
    server_config: ServerConfig,
    start: Option<TimeValue>,
    sender: Sender<GameTimerEvent>,
    rolling_average: RollingAverage,
    observer: Observer,
    new_timer_id: TimerId,
    new_timer_sender: Sender<TimerServiceEvent<GameTimerCreationCallBack, GameTimerCallBack>>
}

impl<Observer: GameTimerObserverTrait> GameTimer<Observer> {

    pub fn new(
            server_config: ServerConfig,
            rolling_average_size: usize,
            observer: Observer,
            sender: Sender<GameTimerEvent>) -> Self {

        let mut timerService = TimeService::<GameTimerCreationCallBack, GameTimerCallBack>::new();
        let call_back = GameTimerCallBack {
            sender: sender.clone()
        };

        let timer_id = timerService.create_timer(call_back, None);

        let new_timer_sender = ThreadBuilder::new()
            .name("NewTimerThread")
            .spawn_event_handler(timerService, AsyncJoin::log_async_join)
            .unwrap();

        GameTimer{
            timer: Timer::new(),
            server_config,
            start: None,
            sender,
            rolling_average: RollingAverage::new(rolling_average_size),
            observer,
            new_timer_id: timer_id,
            new_timer_sender
        }
    }

    fn start_ticking(&mut self) {

        info!("Starting timer with duration {:?}", self.server_config.get_step_duration());

        let now = TimeValue::now();

        self.start = Some(now.add(self.server_config.get_step_duration()));

        let sender_clone = self.sender.clone();

        let schedule = Schedule::Repeating(self.start.unwrap(), self.server_config.get_step_duration());
        self.new_timer_sender.send_event(TimerServiceEvent::RescheduleTimer(self.new_timer_id, Some(schedule))).unwrap();
    }

    fn tick(&self, tick_time_value: TimeValue) {

        trace!("Handling Tick from {:?}", tick_time_value);

        //TODO: tick_time_value is the value from the remote thread, this value gets older and older as the event makes its way through the queue
        //TODO: How much of this can move into the other thread?
        let time_message = TimeMessage::new(
            self.start.clone().unwrap(),
            self.server_config.get_step_duration(),
            tick_time_value);

        if time_message.get_lateness() > TICK_LATENESS_WARN_DURATION {
            warn!("High tick Lateness: {:?}", time_message.get_lateness());
        }

        self.observer.on_time_message(time_message.clone());
    }

    fn on_time_message(&mut self, time_message: TimeReceived<TimeMessage>) {
        trace!("Handling TimeMessage: {:?}", time_message);

        let step_duration = self.server_config.get_step_duration();

        //Calculate the start time of the remote clock in local time and add it to the rolling average
        let remote_start = time_message.get_time_received()
            .subtract(time_message.get().get_lateness())
            .subtract(step_duration * time_message.get().get_step() as f64);

        self.rolling_average.add_value(remote_start.get_seconds_since_epoch());

        let average = self.rolling_average.get_average();

        if self.start.is_none() ||
            (self.start.unwrap().get_seconds_since_epoch() - average).abs() > 1.0  {

            if self.start.is_none() {
                info!("Start client clock from signal from server clock.");
            } else {
                let error = self.start.unwrap().get_seconds_since_epoch() - average;
                if error > CLIENT_ERROR_WARN_DURATION.get_seconds() {
                    warn!("High client error (millis): {:?}", error);
                }
            }

            self.start = Some(TimeValue::from_seconds_since_epoch(average));

            let next_tick = self.start.unwrap()
                .add(step_duration * ((TimeValue::now().duration_since(&self.start.unwrap()) / step_duration)
                    .floor() as f64 + 1.0));

            let schedule = Schedule::Repeating(self.start.unwrap(), self.server_config.get_step_duration());
            self.new_timer_sender.send_event(TimerServiceEvent::RescheduleTimer(self.new_timer_id, Some(schedule))).unwrap();
        }
    }
}

impl<Observer: GameTimerObserverTrait> EventHandlerTrait for GameTimer<Observer> {
    type Event = GameTimerEvent;
    type ThreadReturn = ();

    fn on_channel_event(mut self, channel_event: ChannelEvent<Self::Event>) -> ChannelEventResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(_, event) => {
                match event {
                    StartTickingEvent => self.start_ticking(),
                    TickEvent(tick_time_value) => self.tick(tick_time_value),
                    TimeMessageEvent(time_message) => self.on_time_message(time_message)
                };

                Continue(TryForNextEvent(self))
            }
            ChannelEvent::Timeout => Continue(WaitForNextEvent(self)),
            ChannelEvent::ChannelEmpty => Continue(WaitForNextEvent(self)),
            ChannelEvent::ChannelDisconnected => Break(())
        }
    }

    fn on_stop(self, _: ReceiveMetaData) -> Self::ThreadReturn { () }
}

//TODO: move to its own file
struct GameTimerCallBack {
    sender: Sender<GameTimerEvent>
}

impl TimerCallBack for GameTimerCallBack {
    fn tick(&mut self) {
        info!("new timer yo");
        self.sender.send_event(TickEvent(TimeValue::now())).unwrap();
    }
}

struct GameTimerCreationCallBack {
}

impl TimerCreationCallBack for GameTimerCreationCallBack {
    fn timer_created(self, timer_id: &TimerId) {
        warn!("Timer Created: {:?}", timer_id);
    }
}