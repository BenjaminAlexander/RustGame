use std::sync::{Arc, Mutex};
use commons::factory::FactoryTrait;
use commons::threading::{AsyncJoin, ThreadBuilder};
use commons::time::TimeDuration;
use commons::time::timerservice::{Schedule, TimerCallBack, TimerCreationCallBack, TimerId, TimerServiceEvent, TimeService};
use test_utils::singlethreaded::eventhandling::EventHandlerHolder;
use test_utils::singlethreaded::{SingleThreadedFactory, TimeQueue};

#[test]
fn timer_service_test() {

    let five_seconds = TimeDuration::from_seconds(5.0);

    let factory = SingleThreadedFactory::new();
    let queue = TimeQueue::new(factory.clone());

    let timer_service = TimeService::<SingleThreadedFactory, Box<dyn TimerCreationCallBack>, Box<dyn TimerCallBack>>::new(factory.clone());

    let thread_builder = ThreadBuilder::new();

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
        //panic!();
    });

    let time_value = factory.now().add(five_seconds);

    event_handler_holder.send_event(TimerServiceEvent::CreateTimer(timer_creation_call_back, timer_tick_call_back, Some(Schedule::Once(time_value))));

    assert_eq!(None, *timer_id_cell.lock().unwrap());

    TimeQueue::run_events(&queue);

    assert_ne!(None, *timer_id_cell.lock().unwrap());

    //assert_eq!(0, *tick_count_cell.lock().unwrap());
}