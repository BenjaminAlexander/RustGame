use std::{backtrace::Backtrace, panic};

use commons::logging::LoggingConfigBuilder;
use log::{error, LevelFilter};

pub fn setup_test_logging() {

    //TODO: set up this logging in non-test code
    let orig_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        error!("panic occurred: {panic_info}\n{}", Backtrace::force_capture());
        orig_hook(panic_info);
    }));

    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);
}
