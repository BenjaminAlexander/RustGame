use commons::real_time::FactoryTrait;
use commons::logging::LoggingConfigBuilder;
use commons::real_time::simulation::SingleThreadedFactory;
use commons::time::timerservice::{
    IdleTimerService,
    Schedule,
    TimerId,
};
use commons::time::TimeDuration;
use log::LevelFilter;
use std::ops::Add;
use std::sync::{
    Arc,
    Mutex,
};
use test_utils::utils::Counter;

#[test]
fn timer_service_test() {
    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Trace);

    let two_seconds = TimeDuration::from_secs_f64(2.0);
    let five_seconds = TimeDuration::from_secs_f64(5.0);
    let seven_seconds = two_seconds.add(&five_seconds);

    let factory = SingleThreadedFactory::new();

    let timer_service = IdleTimerService::new(factory.clone()).start().unwrap();

    let timer_id_cell = Arc::new(Mutex::new(None::<TimerId>));
    let tick_count_cell = Counter::new(0);

    let timer_id_cell_clone = timer_id_cell.clone();
    let timer_creation_call_back = Box::new(move |timer_id: &TimerId| {
        *timer_id_cell_clone.lock().unwrap() = Some(*timer_id);
    });

    let tick_count_cell_clone = tick_count_cell.clone();
    let timer_tick_call_back = Box::new(move || {
        tick_count_cell_clone.increment();
    });

    let time_value = factory.get_time_source().now().add(&five_seconds);

    timer_service
        .create_timer(
            timer_creation_call_back,
            timer_tick_call_back,
            Schedule::Once(time_value),
        )
        .unwrap();

    assert_eq!(None, *timer_id_cell.lock().unwrap());

    factory.get_time_queue().run_events();

    assert_ne!(None, *timer_id_cell.lock().unwrap());
    assert_eq!(0, tick_count_cell.get());

    factory.get_time_queue().advance_time_until(time_value);
    assert_eq!(1, tick_count_cell.get());

    factory
        .get_time_queue()
        .advance_time_for_duration(five_seconds);
    factory
        .get_time_queue()
        .advance_time_for_duration(five_seconds);
    assert_eq!(1, tick_count_cell.get());

    let new_schedule = Schedule::Repeating(
        factory.get_time_source().now().add(&seven_seconds),
        five_seconds,
    );
    timer_service
        .reschedule_timer(timer_id_cell.lock().unwrap().unwrap(), new_schedule)
        .unwrap();

    factory.get_time_queue().run_events();
    assert_eq!(1, tick_count_cell.get());

    factory
        .get_time_queue()
        .advance_time_for_duration(five_seconds);
    assert_eq!(1, tick_count_cell.get());

    factory
        .get_time_queue()
        .advance_time_for_duration(two_seconds);
    assert_eq!(2, tick_count_cell.get());

    factory
        .get_time_queue()
        .advance_time_for_duration(five_seconds);
    assert_eq!(3, tick_count_cell.get());

    factory.get_time_queue().run_events();
}
