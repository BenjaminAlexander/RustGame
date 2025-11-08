use log::info;

pub struct AsyncJoin<T> {
    thread_name: String,
    result: T,
}

impl<T> AsyncJoin<T> {
    pub fn log_async_join(self) {
        info!("Thread Join from: {:?}", self.get_thread_name());
    }

    pub fn new(thread_name: String, result: T) -> Self {
        return Self {
            thread_name,
            result,
        };
    }

    pub fn get_thread_name(&self) -> &String {
        return &self.thread_name;
    }

    pub fn get_result(&self) -> &T {
        return &self.result;
    }
}
