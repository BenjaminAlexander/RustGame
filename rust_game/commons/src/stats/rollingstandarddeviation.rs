use crate::stats::RollingAverage;

pub struct RollingStandardDeviation {

    rolling_average: RollingAverage,
    sum_of_squares: f64
}

impl RollingStandardDeviation {

    pub fn new(size: usize) -> Self {
        Self {
            rolling_average: RollingAverage::new(size),
            sum_of_squares: 0 as f64
        }
    }

    pub fn add_value(&mut self, value: f64) -> Option<f64> {

        let old_average = self.rolling_average.get_average();
        let removed_value_option = self.rolling_average.add_value(value);
        let new_average = self.rolling_average.get_average();

        if let Some(removed_value) = removed_value_option.as_ref() {

            //Old and new counts are the same since a value is being removed
            let old_variance = self.sum_of_squares / (self.count() as f64);
            let new_variance = old_variance + (value - removed_value) * (value - new_average + removed_value - old_average) / (self.count() as f64);

            self.sum_of_squares = new_variance * (self.count() as f64);

        } else {
            self.sum_of_squares = self.sum_of_squares + (value - old_average) * (value - new_average);
        }

        return removed_value_option;
    }

    pub fn get_average(&self) -> f64 {
        return self.rolling_average.get_average();
    }

    pub fn get_variance(&self) -> f64 {
        return self.sum_of_squares / (self.count() as f64);
    }

    pub fn get_standard_deviation(&self) -> f64 {
        return self.get_variance().sqrt();
    }

    pub fn count(&self) -> usize {
        return self.rolling_average.count();
    }

    pub fn is_full(&self) -> bool {
        return self.rolling_average.is_full();
    }
}