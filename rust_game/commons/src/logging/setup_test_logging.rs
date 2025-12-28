use {
    crate::logging::LoggingConfigBuilder,
    log::{
        LevelFilter,
        error,
    },
    std::{
        backtrace::Backtrace,
        panic,
    },
};

pub fn setup_test_logging() {
    let orig_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        error!(
            "panic occurred: {panic_info}\n{}",
            Backtrace::force_capture()
        );
        orig_hook(panic_info);
    }));

    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);
}
