use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Logger, Root};

pub fn initLogging() {
    let stdout = ConsoleAppender::builder().build();

    let requests = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build("log/requests.log")
        .unwrap();

    let consoleAppender = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{h({l})}][{d}][{T}-{I}][{M} {f}:{L}]{n}{m}{n}{n}")))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("consoleAppender", Box::new(consoleAppender)))
        .appender(Appender::builder().build("requests", Box::new(requests)))
        .logger(Logger::builder().build("app::backend::db", LevelFilter::Info))
        .logger(Logger::builder()
            .appender("requests")
            .additive(false)
            .build("app::requests", LevelFilter::Info))
        //This is the level:
        .build(Root::builder().appender("consoleAppender").build(LevelFilter::Trace))
        .unwrap();

    let handle = log4rs::init_config(config).unwrap();

    println!("Logging is set up");
}