use std::time::Duration;

use commons::{real_time::SingleThreadExecutor, test_utils::AsyncExpects};

#[test]
fn test_executor_wait() {
    let executor = SingleThreadExecutor::new();
    let async_expects = AsyncExpects::new();
    let expect = async_expects.new_async_expect("an event", ());

    std::thread::sleep(Duration::from_millis(100));

    executor.execute_function_or_panic(move || {
        expect.set_actual(());
    });

    async_expects.wait_for_all();
}
