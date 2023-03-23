use commons::time::TimeValue;
use test_utils::time::SimulatedTimeSource;

#[test]
fn test_simulated_time_provider() {

    let time_source = SimulatedTimeSource::new();
    let time_source_clone = time_source.clone();

    assert_eq!(time_source.now(), TimeValue::from_seconds_since_epoch(0.0));

    let new_time = TimeValue::from_seconds_since_epoch(1234.4);

    time_source.set_simulated_time(new_time);

    assert_eq!(time_source_clone.now(), new_time);
}