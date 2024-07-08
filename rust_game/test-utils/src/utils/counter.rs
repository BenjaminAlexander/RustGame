use std::sync::{
    Arc,
    Mutex,
};

#[derive(Clone)]
pub struct Counter {
    count: Arc<Mutex<usize>>,
}

impl Counter {
    pub fn new(value: usize) -> Self {
        return Self {
            count: Arc::new(Mutex::new(value)),
        };
    }

    pub fn increment(&self) {
        let mut count = self.count.lock().unwrap();
        *count = *count + 1;
    }

    pub fn get(&self) -> usize {
        return *self.count.lock().unwrap();
    }

    pub fn set(&self, value: usize) {
        *self.count.lock().unwrap() = value;
    }
}

impl Default for Counter {
    fn default() -> Self {
        return Self::new(0);
    }
}
