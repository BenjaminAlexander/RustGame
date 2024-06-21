pub struct TotalAverage {
    count: usize,
    sum: f64,
}

impl TotalAverage {
    pub fn new() -> Self {
        Self { count: 0, sum: 0.0 }
    }

    pub fn add_value(&mut self, value: f64) {
        self.sum = self.sum + value;
        self.count = self.count + 1;
    }

    pub fn get_average(&self) -> f64 {
        return if self.count == 0 {
            0.0
        } else {
            self.sum / (self.count as f64)
        };
    }

    pub fn count(&self) -> usize {
        return self.count;
    }
}
