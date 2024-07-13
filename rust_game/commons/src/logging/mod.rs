mod loggingconfigbuilder;

use log::error;
use std::fmt::Debug;
pub use loggingconfigbuilder::LoggingConfigBuilder;

pub fn unwrap_or_log_panic<T, U: Debug>(result: Result<T, U>) -> T {

    match result {
        Ok(value) => return value,
        Err(error) => {
            error!("Failed to unwrap: {error:?}");
            panic!("Failed to unwrap: {error:?}");
        },
    }
}
