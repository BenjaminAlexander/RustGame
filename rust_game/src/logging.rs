use std::path::Path;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use log::*;

const CONSOLE_APPENDER: &str = "console_appender";
const FILE_APPENDER: &str = "file_appender";
const PATTERN: &str = "[{h({l})}][{T}-{I}][{M} {f}:{L}][{d}]{n}{m}{n}{n}";
pub fn init_logging<P: AsRef<Path>>(log_file_path: P) {

    let file_appender = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(PATTERN)))
        .build(log_file_path)
        .unwrap();

    let console_appender = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(PATTERN)))
        .build();

    let config = Config::builder()
        //.appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build(CONSOLE_APPENDER, Box::new(console_appender)))
        .appender(Appender::builder().build(FILE_APPENDER, Box::new(file_appender)))
        .build(Root::builder()
            .appender(CONSOLE_APPENDER)
            .appender(FILE_APPENDER)
            .build(LevelFilter::Info)) //This is the level
        .unwrap();

    log4rs::init_config(config).unwrap();

    info!("Logging is set up");
}