use commons::single_threaded_simulator::SingleThreadedFactory;
use commons::time::TimeDuration;
use test_utils::utils::Counter;

#[test]
fn test_queue() {
    let factory = SingleThreadedFactory::new();

    let cell = Counter::new(0);

    let five_seconds = TimeDuration::from_secs_f64(5.0);

    let cell_clone = cell.clone();
    let queue_clone = factory.get_time_queue().clone();
    factory
        .get_time_queue()
        .add_event_at_duration_from_now(five_seconds.mul_f64(2.0), move || {
            cell_clone.set(1);

            queue_clone.add_event_at_duration_from_now(five_seconds, move || {
                cell_clone.set(2);
            });
        });

    let cell_clone = cell.clone();
    let id_to_remove = factory.get_time_queue().add_event_at_duration_from_now(
        five_seconds.mul_f64(4.0),
        move || {
            cell_clone.set(3);
        },
    );

    assert_eq!(cell.get(), 0);

    factory
        .get_time_queue()
        .advance_time_for_duration(five_seconds);
    assert_eq!(cell.get(), 0);

    factory
        .get_time_queue()
        .advance_time_for_duration(five_seconds);
    assert_eq!(cell.get(), 1);

    factory
        .get_time_queue()
        .advance_time_for_duration(five_seconds);
    assert_eq!(cell.get(), 2);

    factory.get_time_queue().remove_event(id_to_remove);

    factory
        .get_time_queue()
        .advance_time_for_duration(five_seconds);
    assert_eq!(cell.get(), 2);
}
