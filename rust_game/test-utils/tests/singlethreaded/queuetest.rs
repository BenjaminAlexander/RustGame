use std::cell::Cell;
use std::rc::Rc;
use commons::time::{TimeDuration, TimeSource};
use test_utils::singlethreaded::Queue;
use test_utils::time::SimulatedTimeProvider;

#[test]
fn test_simulated_time_provider() {

    SimulatedTimeProvider::reset();

    let cell = Rc::new(Cell::new(0));

    let queue = Queue::new();
    let five_seconds = TimeDuration::from_seconds(5.0);

    let cell_clone = cell.clone();
    let queue_clone = queue.clone();
    Queue::add_event_duration_from_now(&queue, five_seconds * 2.0, move || {
        cell_clone.set(1);

        Queue::add_event_duration_from_now(&queue_clone, five_seconds, move || {
            cell_clone.set(2);
        });
    });

    let cell_clone = cell.clone();
    let id_to_remove = Queue::add_event_duration_from_now(&queue, five_seconds * 4.0, move || {
        cell_clone.set(3);
    });

    assert_eq!(cell.get(), 0);

    Queue::advance_time_for_duration(&queue, five_seconds);
    assert_eq!(cell.get(), 0);

    Queue::advance_time_for_duration(&queue, five_seconds);
    assert_eq!(cell.get(), 1);

    Queue::advance_time_for_duration(&queue, five_seconds);
    assert_eq!(cell.get(), 2);

    Queue::remove_event(&queue, id_to_remove);

    Queue::advance_time_for_duration(&queue, five_seconds);
    assert_eq!(cell.get(), 2);
}