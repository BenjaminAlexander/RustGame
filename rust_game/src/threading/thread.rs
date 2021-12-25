use std::io;
use std::thread::{Builder, JoinHandle};

pub trait Thread<T> : Sized + Send + 'static
    where T: Send + 'static {

    fn build(self) -> ThreadBuilder<T> {
        let x = Box::new(||{
            self.run()
        });
        ThreadBuilder{thread: x, builder: Builder::new()}
    }

    fn run(self) -> T;
}

pub struct ThreadBuilder<T>
    where T: Send + 'static {

    thread: Box<dyn FnOnce() -> T + Send>,
    builder: Builder
}

impl<T> ThreadBuilder<T>
    where T: Send + 'static {

    pub fn name(mut self, name: &str) -> Self {
        self.builder = self.builder.name(name.to_string());
        self
    }

    pub fn start(self) -> io::Result<JoinHandle<T>> {
        let builder = self.builder;
        let thread = self.thread;

        builder.spawn(thread)
    }
}