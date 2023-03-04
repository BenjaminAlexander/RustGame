use log::info;
use super::ThreadBuilder;

pub struct AsyncJoin<T> {
    thread_builder: ThreadBuilder,
    result: T
}

impl<T> AsyncJoin<T> {

    pub fn log_async_join(self) {
        info!("Thread Join from: {:?}", self.get_thread_name());
    }

    pub fn new(thread_builder: ThreadBuilder, result: T) -> Self {
        return Self {
            thread_builder,
            result
        }
    }

    pub fn get_thread_name(&self) -> Option<&String> {
        return self.thread_builder.get_name();
    }

    pub fn get_result(&self) -> &T {
        return &self.result;
    }
}