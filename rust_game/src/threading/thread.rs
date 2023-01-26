use std::io::Result;
use std::thread::{Builder, JoinHandle};
use log::info;

pub fn old_build_thread<T: Thread>(thread: T) -> OldThreadBuilder<T> {
    return OldThreadBuilder {
        thread,
        builder: Builder::new()
    };
}

pub trait Thread : Sized + Send + 'static {

    type ReturnType: Send + 'static;

    fn run(self) -> Self::ReturnType;
}

pub trait OldThreadBuilderTrait {
    //TODO: rename
    type StartResultType;

    fn name(self, name: &str) -> Self;

    fn start(self) -> Self::StartResultType;
}

pub struct OldThreadBuilder<ThreadType: Thread> {
    thread: ThreadType,
    builder: Builder
}

impl<T: Thread> OldThreadBuilderTrait for OldThreadBuilder<T> {
    type StartResultType = Result<JoinHandle<T::ReturnType>>;

    fn name(mut self, name: &str) -> Self {
        self.builder = self.builder.name(name.to_string());
        return self;
    }

    fn start(self) -> Result<JoinHandle<T::ReturnType>> {
        let builder = self.builder;
        let thread = self.thread;

        return builder.spawn(||{

            info!("Thread Starting");

            let return_value = thread.run();

            info!("Thread Ending");

            return return_value;
        });
    }
}