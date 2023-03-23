use std::ops::Add;
use std::sync::{Arc, Mutex};
use log::LevelFilter;
use commons::factory::FactoryTrait;
use commons::logging::LoggingConfigBuilder;
use commons::threading::{AsyncJoin, ThreadBuilder};
use commons::time::TimeDuration;
use commons::time::timerservice::{Schedule, TimerCallBack, TimerCreationCallBack, TimerId, TimerServiceEvent, TimeService};
use test_utils::singlethreaded::eventhandling::EventHandlerHolder;
use test_utils::singlethreaded::{SingleThreadedFactory, TimeQueue};

#[test]
fn timer_service_test() {

    LoggingConfigBuilder::new().add_console_appender().init(LevelFilter::Trace);

    let two_seconds = TimeDuration::from_seconds(2.0);
    let five_seconds = TimeDuration::from_seconds(5.0);
    let seven_seconds = two_seconds.add(five_seconds);

    let factory = SingleThreadedFactory::new();
    let queue = TimeQueue::new(factory.clone());

    let timer_service = TimeService::<SingleThreadedFactory, Box<dyn TimerCreationCallBack>, Box<dyn TimerCallBack>>::new(factory.clone());

    let thread_builder = ThreadBuilder::new(factory.clone());

    let timer_id_cell = Arc::new(Mutex::new(None::<TimerId>));
    let tick_count_cell = Arc::new(Mutex::new(0));

    let event_handler_holder = EventHandlerHolder::new(queue.clone(), thread_builder, timer_service, AsyncJoin::log_async_join);

    let timer_id_cell_clone = timer_id_cell.clone();
    let timer_creation_call_back = Box::new(move |timer_id: &TimerId| {
        *timer_id_cell_clone.lock().unwrap() = Some(*timer_id);
    });

    let tick_count_cell_clone = tick_count_cell.clone();
    let timer_tick_call_back = Box::new(move || {
        let mut tick_count = tick_count_cell_clone.lock().unwrap();
        *tick_count = *tick_count + 1;
    });

    let time_value = factory.now().add(five_seconds);

    event_handler_holder.send_event(TimerServiceEvent::CreateTimer(timer_creation_call_back, timer_tick_call_back, Some(Schedule::Once(time_value))));

    assert_eq!(None, *timer_id_cell.lock().unwrap());

    queue.run_events();

    assert_ne!(None, *timer_id_cell.lock().unwrap());
    assert_eq!(0, *tick_count_cell.lock().unwrap());

    queue.advance_time_until(time_value);
    assert_eq!(1, *tick_count_cell.lock().unwrap());

    queue.advance_time_for_duration(five_seconds);
    queue.advance_time_for_duration(five_seconds);
    assert_eq!(1, *tick_count_cell.lock().unwrap());


    let new_schedule = Schedule::Repeating(factory.now().add(seven_seconds), five_seconds);
    event_handler_holder.send_event(TimerServiceEvent::RescheduleTimer(timer_id_cell.lock().unwrap().unwrap(), Some(new_schedule)));
    queue.run_events();
    assert_eq!(1, *tick_count_cell.lock().unwrap());

    queue.advance_time_for_duration(five_seconds);
    assert_eq!(1, *tick_count_cell.lock().unwrap());

    queue.advance_time_for_duration(two_seconds);
    assert_eq!(2, *tick_count_cell.lock().unwrap());

    queue.advance_time_for_duration(five_seconds);
    assert_eq!(3, *tick_count_cell.lock().unwrap());
}