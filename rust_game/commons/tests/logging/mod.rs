use commons::{logging::LoggingConfigBuilder, test_utils::with_test_temp_dir};
use log::{
    info,
    LevelFilter,
};

#[test]
fn test_log_to_file() {
    with_test_temp_dir(|tmp_dir| {
        let mut log_file_path = tmp_dir.to_path_buf();
        log_file_path.push("log.txt");

        LoggingConfigBuilder::new()
            .add_console_appender()
            .add_file_appender(&log_file_path)
            .init(LevelFilter::Info);

        info!("This is a message");

        let metadata = std::fs::metadata(log_file_path).unwrap();
        assert!(metadata.is_file());
    });
}
