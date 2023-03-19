use commons::time::{TimeSource, TimeValue};
use test_utils::time::SimulatedTimeProvider;

#[test]
fn test_simulated_time_provider() {
    assert_eq!(SimulatedTimeProvider::now(), TimeValue::from_seconds_since_epoch(0.0));

    let new_time = TimeValue::from_seconds_since_epoch(1234.4);

    SimulatedTimeProvider::set_simulated_time(&new_time);

    assert_eq!(SimulatedTimeProvider::now(), new_time);
}