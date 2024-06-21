use super::ThreadBuilder;
use crate::factory::FactoryTrait;
use log::info;

pub struct AsyncJoin<Factory: FactoryTrait, T> {
    thread_builder: ThreadBuilder<Factory>,
    result: T,
}

impl<Factory: FactoryTrait, T> AsyncJoin<Factory, T> {
    pub fn log_async_join(self) {
        info!("Thread Join from: {:?}", self.get_thread_name());
    }

    pub fn new(thread_builder: ThreadBuilder<Factory>, result: T) -> Self {
        return Self {
            thread_builder,
            result,
        };
    }

    pub fn get_thread_name(&self) -> Option<&String> {
        return self.thread_builder.get_name();
    }

    pub fn get_result(&self) -> &T {
        return &self.result;
    }
}
