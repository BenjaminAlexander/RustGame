use crate::stats::TotalAverage;

pub struct TotalStandardDeviation {
    average: TotalAverage,
    sum_of_squares: f64,
}

impl TotalStandardDeviation {
    pub fn new() -> Self {
        Self {
            average: TotalAverage::new(),
            sum_of_squares: 0 as f64,
        }
    }

    pub fn add_value(&mut self, value: f64) {
        let old_average = self.average.get_average();
        self.average.add_value(value);
        let new_average = self.average.get_average();
        self.sum_of_squares = self.sum_of_squares + (value - old_average) * (value - new_average);
    }

    pub fn get_average(&self) -> f64 {
        return self.average.get_average();
    }

    pub fn get_variance(&self) -> f64 {
        return self.sum_of_squares / (self.count() as f64);
    }

    pub fn get_standard_deviation(&self) -> f64 {
        return self.get_variance().sqrt();
    }

    pub fn count(&self) -> usize {
        return self.average.count();
    }
}
