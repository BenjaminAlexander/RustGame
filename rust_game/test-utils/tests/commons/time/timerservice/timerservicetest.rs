use std::cell::Cell;
use std::rc::Rc;
use commons::threading::{AsyncJoin, ThreadBuilder};
use commons::time::{TimeDuration, TimeSource};
use commons::time::timerservice::{Schedule, TimerCallBack, TimerCreationCallBack, TimerId, TimerServiceEvent, TimeService};
use test_utils::singlethreaded::eventhandling::EventHandlerHolder;
use test_utils::singlethreaded::Queue;
use test_utils::time::SimulatedTimeProvider;

#[test]
fn test_simulated_time_provider() {

    let five_seconds = TimeDuration::from_seconds(5.0);

    SimulatedTimeProvider::reset();

    let queue = Queue::new();

    let timer_service = TimeService::<Box<dyn TimerCreationCallBack>, Box<dyn TimerCallBack>>::new();

    let thread_builder = ThreadBuilder::new();

    let timer_id_cell = Rc::new(Cell::new(None::<TimerId>));
    let tick_count_cell = Rc::new(Cell::new(0));

    let x = EventHandlerHolder::new(queue.clone(), thread_builder, timer_service, AsyncJoin::log_async_join);

    let timer_id_cell_clone = timer_id_cell.clone();
    let timer_creation_call_back = Box::new(|timer_id: &TimerId| {
        timer_id_cell_clone.set(Some(*timer_id));
    });

    let tick_count_cell_clone = tick_count_cell.clone();
    let timer_tick_call_back = Box::new(|| {
        let tick_count = tick_count_cell_clone.get() + 1;
        tick_count_cell_clone.set(tick_count);
    });

    let time_value = SimulatedTimeProvider::now().add(five_seconds);

    //x.send(TimerServiceEvent::CreateTimer(timer_creation_call_back, timer_tick_call_back, Some(Schedule::Once(time_value))));

    assert_eq!(None, timer_id_cell.get());
}