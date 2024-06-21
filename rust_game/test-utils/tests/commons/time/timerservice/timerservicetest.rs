use std::ops::Add;
use std::sync::{Arc, Mutex};
use log::LevelFilter;
use commons::factory::FactoryTrait;
use commons::logging::LoggingConfigBuilder;
use commons::threading::eventhandling::EventSenderTrait;
use commons::time::TimeDuration;
use commons::time::timerservice::{Schedule, TimerCallBack, TimerCreationCallBack, TimerId, TimerServiceEvent, TimeService};
use test_utils::singlethreaded::SingleThreadedFactory;
use test_utils::utils::Counter;

type TimerServiceAlias = TimeService<SingleThreadedFactory, Box<dyn TimerCreationCallBack>, Box<dyn TimerCallBack>>;

#[test]
fn timer_service_test() {

    LoggingConfigBuilder::new().add_console_appender().init(LevelFilter::Trace);

    let two_seconds = TimeDuration::from_secs_f64(2.0);
    let five_seconds = TimeDuration::from_secs_f64(5.0);
    let seven_seconds = two_seconds.add(&five_seconds);

    let factory = SingleThreadedFactory::new();

    let timer_service = TimerServiceAlias::new(factory.clone());

    let timer_id_cell = Arc::new(Mutex::new(None::<TimerId>));
    let tick_count_cell = Counter::new(0);
    let join_counter = Counter::new(0);

    let join_counter_clone = join_counter.clone();
    let channel_thread_builder = factory.new_thread_builder().build_channel_for_event_handler::<TimerServiceAlias>();

    let timer_id_cell_clone = timer_id_cell.clone();
    let timer_creation_call_back = Box::new(move |timer_id: &TimerId| {
        *timer_id_cell_clone.lock().unwrap() = Some(*timer_id);
    });

    let tick_count_cell_clone = tick_count_cell.clone();
    let timer_tick_call_back = Box::new(move || {
        tick_count_cell_clone.increment();
    });

    let time_value = factory.now().add(&five_seconds);
    channel_thread_builder.get_sender().send_event(TimerServiceEvent::CreateTimer(timer_creation_call_back, timer_tick_call_back, Some(Schedule::Once(time_value)))).unwrap();

    let sender = channel_thread_builder.spawn_event_handler(timer_service, move |_async_join|{
        join_counter_clone.increment();
    }).unwrap();

    assert_eq!(None, *timer_id_cell.lock().unwrap());

    factory.get_time_queue().run_events();

    assert_ne!(None, *timer_id_cell.lock().unwrap());
    assert_eq!(0, tick_count_cell.get());

    factory.get_time_queue().advance_time_until(time_value);
    assert_eq!(1, tick_count_cell.get());

    factory.get_time_queue().advance_time_for_duration(five_seconds);
    factory.get_time_queue().advance_time_for_duration(five_seconds);
    assert_eq!(1, tick_count_cell.get());


    let new_schedule = Schedule::Repeating(factory.now().add(&seven_seconds), five_seconds);
    sender.send_event(TimerServiceEvent::RescheduleTimer(timer_id_cell.lock().unwrap().unwrap(), Some(new_schedule))).unwrap();
    factory.get_time_queue().run_events();
    assert_eq!(1, tick_count_cell.get());

    factory.get_time_queue().advance_time_for_duration(five_seconds);
    assert_eq!(1, tick_count_cell.get());

    factory.get_time_queue().advance_time_for_duration(two_seconds);
    assert_eq!(2, tick_count_cell.get());

    factory.get_time_queue().advance_time_for_duration(five_seconds);
    assert_eq!(3, tick_count_cell.get());

    assert_eq!(0, join_counter.get());
    drop(sender);
    assert_eq!(0, join_counter.get());
    factory.get_time_queue().run_events();
    assert_eq!(1, join_counter.get());
}
