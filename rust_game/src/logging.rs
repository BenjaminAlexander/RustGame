use std::path::Path;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use log::*;

pub fn init_logging<P: AsRef<Path>>(log_file_path: P) {
    //let stdout = ConsoleAppender::builder().build();

    let file_appender = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{h({l})}][{d}][{T}-{I}][{M} {f}:{L}]{n}{m}{n}{n}")))
        .build(log_file_path)
        .unwrap();

    let console_appender = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{h({l})}][{d}][{T}-{I}][{M} {f}:{L}]{n}{m}{n}{n}")))
        .build();

    let config = Config::builder()
        //.appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("console_appender", Box::new(console_appender)))
        .appender(Appender::builder().build("file_appender", Box::new(file_appender)))
        //.appender(Appender::builder().build("requests", Box::new(requests)))
        //.logger(Logger::builder().build("app::backend::db", LevelFilter::Info))
        //.logger(Logger::builder()
        //    .appender("requests")
        //    .additive(false)
        //    .build("app::requests", LevelFilter::Info))

        //This is the level:
        .build(Root::builder()
            .appender("console_appender")
            .appender("file_appender")
            .build(LevelFilter::Debug))
        .unwrap();

    log4rs::init_config(config).unwrap();

    info!("Logging is set up");
}