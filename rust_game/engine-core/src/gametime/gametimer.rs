use commons::stats::RollingAverage;
use commons::time::{TimeValue, TimeDuration};
use crate::gametime::{TimeMessage, TimeReceived};
use timer::Timer;
use log::{trace, info, warn};
use commons::factory::FactoryTrait;
use crate::server::ServerConfig;
use commons::threading::eventhandling::{EventHandlerSender, EventSenderTrait};
use commons::threading::AsyncJoin;
use commons::time::timerservice::{Schedule, TimerCallBack, TimerCreationCallBack, TimerId, TimerServiceEvent, TimeService};

const TICK_LATENESS_WARN_DURATION: TimeDuration = TimeDuration::from_seconds(0.02);
const CLIENT_ERROR_WARN_DURATION: TimeDuration = TimeDuration::from_seconds(0.02);

pub struct GameTimer<Factory: FactoryTrait, T: TimerCallBack> {
    factory: Factory,
    //TODO: remove timer
    timer: Timer,
    server_config: ServerConfig,
    start: Option<TimeValue>,
    rolling_average: RollingAverage,
    new_timer_id: TimerId,
    new_timer_sender: EventHandlerSender<Factory, TimerServiceEvent<GameTimerCreationCallBack, T>>
}

impl<Factory: FactoryTrait, T: TimerCallBack> GameTimer<Factory, T> {

    pub fn new(
            factory: Factory,
            server_config: ServerConfig,
            rolling_average_size: usize,
            call_back: T) -> Self {

        let mut timer_service = TimeService::<Factory, GameTimerCreationCallBack, T>::new(factory.clone());

        let timer_id = timer_service.create_timer(call_back, None);

        let new_timer_sender = factory.new_thread_builder()
            .name("NewTimerThread")
            .spawn_event_handler(timer_service, AsyncJoin::log_async_join)
            .unwrap();

        return Self {
            factory,
            timer: Timer::new(),
            server_config,
            start: None,
            rolling_average: RollingAverage::new(rolling_average_size),
            new_timer_id: timer_id,
            new_timer_sender
        };
    }

    pub fn start_ticking(&mut self) {

        info!("Starting timer with duration {:?}", self.server_config.get_step_duration());

        let now = self.factory.now();

        self.start = Some(now.add(self.server_config.get_step_duration()));

        let schedule = Schedule::Repeating(self.start.unwrap(), self.server_config.get_step_duration());
        self.new_timer_sender.send_event(TimerServiceEvent::RescheduleTimer(self.new_timer_id, Some(schedule))).unwrap();
    }

    pub fn on_time_message(&mut self, time_message: TimeReceived<TimeMessage>) {
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
                .add(step_duration * ((self.factory.now().duration_since(&self.start.unwrap()) / step_duration)
                    .floor() as f64 + 1.0));

            let schedule = Schedule::Repeating(self.start.unwrap(), self.server_config.get_step_duration());
            self.new_timer_sender.send_event(TimerServiceEvent::RescheduleTimer(self.new_timer_id, Some(schedule))).unwrap();
        }
    }

    pub fn create_timer_message(&self) -> TimeMessage {
        let now = self.factory.now();

        //TODO: tick_time_value is the value from the remote thread, this value gets older and older as the event makes its way through the queue
        //TODO: How much of this can move into the other thread?
        let time_message = TimeMessage::new(
            self.start.clone().unwrap(),
            self.server_config.get_step_duration(),
            now);

        if time_message.get_lateness() > TICK_LATENESS_WARN_DURATION {
            warn!("High tick Lateness: {:?}", time_message.get_lateness());
        }

        return time_message;
    }
}

struct GameTimerCreationCallBack {
}

impl TimerCreationCallBack for GameTimerCreationCallBack {
    fn timer_created(&self, timer_id: &TimerId) {
        warn!("Timer Created: {:?}", timer_id);
    }
}
