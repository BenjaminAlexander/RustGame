use std::fmt::Debug;

use log::error;

pub fn simplify_result<T, U>(result: Result<T, U>) -> Result<T, ()> {
    return match result {
        Ok(t) => Ok(t),
        Err(_) => Err(()),
    };
}

pub fn log_error<T: Debug>(err: T) -> () {
    error!("Result is error: {:?}", err);
    ()
}
