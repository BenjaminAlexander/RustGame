use commons::{logging::LoggingConfigBuilder, threading::SingleThreadExecutor};
use log::LevelFilter;
use test_utils::assert::{wait_for_all_async_expects, AsyncExpect};

#[test]
fn test_async_expect() {

    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);

    let executor = SingleThreadExecutor::new();

    let expect_to_start_1 = AsyncExpect::new("Expect to start Runnable (1)", true);
    let expect_to_start_2 = AsyncExpect::new("Expect to start Runnable (2)", true);
    let expect_to_end_1 = AsyncExpect::new("Expect to end test (1)", true);
    let expect_to_end_2 = AsyncExpect::new("Expect to end test (2)", true);
    
    let expect_to_start_1_clone = expect_to_start_1.clone();
    let expect_to_start_2_clone = expect_to_start_2.clone();
    let expect_to_end_1_clone = expect_to_end_1.clone();
    let expect_to_end_2_clone = expect_to_end_2.clone();

    executor.execute_function_or_panic(move ||{

        expect_to_start_1_clone.wait_for();

        expect_to_end_1_clone.set_actual(true);

        expect_to_start_2_clone.wait_for();

        expect_to_end_2_clone.set_actual(true);
    });

    expect_to_start_1.set_actual(true);

    expect_to_end_1.wait_for();

    expect_to_start_2.set_actual(true);

    wait_for_all_async_expects();    
}