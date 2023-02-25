pub struct RollingAverage {
    values: Vec<f64>,
    next_index: usize,
    sum: f64
}

impl RollingAverage {

    pub fn new(size: usize) -> Self {
        Self {
            values: Vec::with_capacity(size),
            next_index: 0,
            sum: 0.0
        }
    }

    pub fn add_value(&mut self, value: f64) -> Option<f64> {

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
            0.0
        } else {
            self.sum / (self.values.len() as f64)
        }
    }

    pub fn count(&self) -> usize {
        return self.values.len();
    }

    pub fn is_full(&self) -> bool {
        return self.values.len() == self.values.capacity();
    }
}