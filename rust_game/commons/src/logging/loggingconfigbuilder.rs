use log::*;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::runtime::{
    ConfigBuilder,
    RootBuilder,
};
use log4rs::config::{
    Appender,
    Config,
    Root,
};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Handle;
use std::path::Path;
use std::sync::Mutex;

const CONSOLE_APPENDER: &str = "console_appender";
const FILE_APPENDER: &str = "file_appender";
const PATTERN: &str = "[{h({l})}][{T}-{I}][{M} {f}:{L}][{d}]{n}{m}{n}{n}";

static LOGGER_HANDLE: Mutex<Option<Handle>> = Mutex::new(None);

pub struct LoggingConfigBuilder {
    config_builder: ConfigBuilder,
    root_builder: RootBuilder,
}

impl LoggingConfigBuilder {
    pub fn new() -> Self {
        return Self {
            config_builder: Config::builder(),
            root_builder: Root::builder(),
        };
    }

    pub fn add_file_appender(mut self, log_file_path: impl AsRef<Path>) -> Self {
        let file_appender = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(PATTERN)))
            .build(log_file_path)
            .unwrap();

        self.config_builder = self
            .config_builder
            .appender(Appender::builder().build(FILE_APPENDER, Box::new(file_appender)));
        self.root_builder = self.root_builder.appender(FILE_APPENDER);
        return self;
    }

    pub fn add_console_appender(mut self) -> Self {
        let console_appender = ConsoleAppender::builder()
            .encoder(Box::new(PatternEncoder::new(PATTERN)))
            .build();

        self.config_builder = self
            .config_builder
            .appender(Appender::builder().build(CONSOLE_APPENDER, Box::new(console_appender)));
        self.root_builder = self.root_builder.appender(CONSOLE_APPENDER);
        return self;
    }

    pub fn init(self, level_filter: LevelFilter) {
        let config = self
            .config_builder
            .build(self.root_builder.build(level_filter))
            .unwrap();

        let mut guard = LOGGER_HANDLE.lock().unwrap();

        match *guard {
            None => {
                *guard = Some(log4rs::init_config(config).unwrap());
                info!("Logging is set up");
            }
            Some(ref handle) => {
                handle.set_config(config);
                info!("Set logging config");
            }
        }
    }
}
