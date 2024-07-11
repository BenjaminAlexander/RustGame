use commons::logging::LoggingConfigBuilder;
use log::LevelFilter;

pub fn setup_test_logging() {
    LoggingConfigBuilder::new()
        .add_console_appender()
        .init(LevelFilter::Info);
}
