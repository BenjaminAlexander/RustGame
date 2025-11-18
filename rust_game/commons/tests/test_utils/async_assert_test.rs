use commons::{
    logging::LoggingConfigBuilder,
    real_time::SingleThreadExecutor, test_utils::AsyncExpects,
};
use log::LevelFilter;
use std::panic::catch_unwind;

#[test]
fn test_async_expect() {
    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);

    let executor = SingleThreadExecutor::new();

    let async_expects = AsyncExpects::new();

    let expect_to_start_1 = async_expects.new_async_expect("Expect to start Runnable (1)", true);
    let expect_to_start_2 = async_expects.new_async_expect("Expect to start Runnable (2)", true);
    let expect_to_end_1 = async_expects.new_async_expect("Expect to end test (1)", true);
    let expect_to_end_2 = async_expects.new_async_expect("Expect to end test (2)", true);

    let expect_to_start_1_clone = expect_to_start_1.clone();
    let expect_to_start_2_clone = expect_to_start_2.clone();
    let expect_to_end_1_clone = expect_to_end_1.clone();
    let expect_to_end_2_clone = expect_to_end_2.clone();

    executor.execute_function_or_panic(move || {
        expect_to_start_1_clone.wait_for();

        expect_to_end_1_clone.set_actual(true);

        expect_to_start_2_clone.wait_for();

        expect_to_end_2_clone.set_actual(true);
    });

    expect_to_start_1.set_actual(true);

    expect_to_end_1.wait_for();

    expect_to_start_2.set_actual(true);

    async_expects.wait_for_all();
}

#[test]
#[should_panic]
fn test_failed_async_expect() {
    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);

    let async_expects = AsyncExpects::new();

    let async_expect = async_expects.new_async_expect("Expect true", true);

    async_expect.set_actual(false);
}

#[test]
#[should_panic]
fn test_duplicate() {
    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);

    let async_expects = AsyncExpects::new();

    let async_expect = async_expects.new_async_expect("Expect true", true);

    async_expect.set_actual(true);

    async_expect.set_actual(true);
}

#[test]
#[should_panic]
fn test_duplicate_async() {
    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);

    let async_expects = AsyncExpects::new();

    let async_expect_to_set_multiple_times = async_expects.new_async_expect("Expect true", true);

    async_expect_to_set_multiple_times.set_actual(true);

    catch_unwind(|| {
        async_expect_to_set_multiple_times.set_actual(true);
    })
    .unwrap_err();

    catch_unwind(|| {
        async_expect_to_set_multiple_times.set_actual(true);
    })
    .unwrap_err();

    async_expects.wait_for_all();
}

#[test]
#[should_panic]
fn test_wait_for_async_panic() {
    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);

    let executor = SingleThreadExecutor::new();

    let async_expects = AsyncExpects::new();

    let expect = async_expects.new_async_expect("Expect true", true);

    let expect_clone = expect.clone();

    executor.execute_function_or_panic(move || {
        expect_clone.set_actual(false);
    });

    expect.wait_for();
}

#[test]
#[should_panic]
fn test_wait_for_all_async_panic() {
    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);

    let executor = SingleThreadExecutor::new();

    let async_expects = AsyncExpects::new();

    let expect = async_expects.new_async_expect("Expect true", true);

    let expect_clone = expect.clone();

    executor.execute_function_or_panic(move || {
        expect_clone.set_actual(false);
    });

    async_expects.wait_for_all();
}
