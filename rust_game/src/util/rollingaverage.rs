use std::ops::{Sub, Add, Div};

pub struct RollingAverage<T>
    where T: Default + Copy + Sub<Output = T> + Add<Output = T> + Div<u64, Output = T> + 'static {

    values: Vec<T>,
    next_index: usize,
    sum: T,
}

impl<T> RollingAverage<T>
    where T: Default + Copy + Sub<Output = T> + Add<Output = T> + Div<u64, Output = T> + 'static {

    pub fn new(size: usize) -> Self {
        RollingAverage {
            values: Vec::with_capacity(size),
            next_index: 0,
            sum: T::default(),
        }
    }

    pub fn add_value(&mut self, value: T) {
        if self.values.len() == self.values.capacity() {
            self.sum = self.sum - self.values[self.next_index];
            self.values[self.next_index] = value;
        } else {
            self.values.insert(self.next_index, value);
        }

        self.sum = self.sum + value;
        self.next_index = (self.next_index + 1) % self.values.capacity();
    }

    pub fn get_average(&self) -> T {
        return if self.values.is_empty() {
            T::default()
        } else {
            self.sum / self.values.len() as u64
        }
    }
}