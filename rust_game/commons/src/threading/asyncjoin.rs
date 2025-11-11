use log::info;

pub struct AsyncJoin<T> {
    result: T,
}

impl<T> AsyncJoin<T> {
    pub fn log_async_join(self) {
        info!("Thread Join");
    }

    pub fn new(result: T) -> Self {
        return Self {
            result,
        };
    }

    pub fn get_result(&self) -> &T {
        return &self.result;
    }
}
