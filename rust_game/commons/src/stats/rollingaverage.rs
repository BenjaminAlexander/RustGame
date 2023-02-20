use std::ops::{Sub, Add};
use num::traits::AsPrimitive;

pub struct RollingAverage<T>
    where T: Default + Copy + Sub<Output = T> + Add<Output = T> + AsPrimitive<f64> + 'static {

    values: Vec<T>,
    next_index: usize,
    sum: T
}

impl<T> RollingAverage<T>
    where T: Default + Copy + Sub<Output = T> + Add<Output = T> + AsPrimitive<f64> + 'static {

    pub fn new(size: usize) -> Self {
        Self {
            values: Vec::with_capacity(size),
            next_index: 0,
            sum: T::default()
        }
    }

    pub fn add_value(&mut self, value: T) -> Option<T> {

        let mut removed_value_option = None;

        if self.values.len() == self.values.capacity() {
            let removed_value = self.values[self.next_index];
            self.values[self.next_index] = value;

            self.sum = self.sum - removed_value;
            removed_value_option = Some(removed_value);

        } else {
            self.values.insert(self.next_index, value);
        }

        self.sum = self.sum + value;
        self.next_index = (self.next_index + 1) % self.values.capacity();

        return removed_value_option;
    }

    pub fn get_average(&self) -> f64 {
        return if self.values.is_empty() {
            f64::default()
        } else {
            <T as AsPrimitive<f64>>::as_(self.sum) / (self.values.len() as f64)
        }
    }

    pub fn count(&self) -> usize {
        return self.values.len();
    }
}