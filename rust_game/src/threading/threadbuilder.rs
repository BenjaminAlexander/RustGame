use std::io::Error;
use std::thread::{Builder, JoinHandle};
use log::info;
use crate::threading::Thread;

pub struct ThreadBuilder {
    name: Option<String>
}

impl ThreadBuilder {

    pub fn new() -> Self {
        return Self {
            name: None
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        return self;
    }

    pub(super) fn spawn_thread<T: Thread>(mut self, thread: T) -> std::io::Result<JoinHandle<T::ReturnType>> {
        let mut builder = Builder::new();

        if let Some(name) = self.name.take() {
            builder = builder.name(name);
        }

        return builder.spawn(||{

            info!("Thread Starting");

            let return_value = thread.run();

            info!("Thread Ending");

            return return_value;
        });
    }
}