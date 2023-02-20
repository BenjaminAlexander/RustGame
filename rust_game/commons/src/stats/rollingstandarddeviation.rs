use std::ops::{Add, Sub};
use num::traits::AsPrimitive;
use crate::stats::RollingAverage;

use log::{error, info};

pub struct RollingStandardDeviation<T>
    where T:
        Default +
        Copy +
        Sub<Output = T> +
        Add<Output = T> +
        AsPrimitive<f64> +
        'static {

    rolling_average: RollingAverage<T>,
    m2: f64
}

impl<T> RollingStandardDeviation<T>
    where T:
        Default +
        Copy +
        Sub<Output = T> +
        Add<Output = T> +
        AsPrimitive<f64> +
        'static {

    pub fn new(size: usize) -> Self {
        Self {
            rolling_average: RollingAverage::new(size),
            m2: 0 as f64
        }
    }

    pub fn add_value(&mut self, value: T) -> Option<T> {

        let old_average = self.rolling_average.get_average();
        let removed_value_option = self.rolling_average.add_value(value);
        let new_average = self.rolling_average.get_average();
        let f64_value = <T as AsPrimitive<f64>>::as_(value);

        info!("f64_value: {:?}", f64_value);

        if let Some(removed_value) = removed_value_option.as_ref() {





            //Old and new counts are the same since a value is being removed
            let f64_removed_value: f64 = <T as AsPrimitive<f64>>::as_(*removed_value);
            info!("f64_removed_value: {:?}", f64_removed_value);

            let old_variance = self.m2 / (self.count() as f64);
            info!("old_variance: {:?}", old_variance);
            let new_variance = old_variance + (f64_value - f64_removed_value) * (f64_value - new_average + f64_removed_value - old_average) / ((self.count() - 1) as f64);
            info!("new_variance: {:?}", new_variance);
            self.m2 = new_variance * (self.count() as f64);
            info!("self.m2: {:?}", self.m2);

        } else {
            self.m2 = self.m2 + (f64_value - old_average) * (f64_value - new_average);
        }

        return removed_value_option;
    }

    pub fn get_average(&self) -> f64 {
        return self.rolling_average.get_average();
    }

    pub fn get_variance(&self) -> f64 {
        return self.m2 / (self.count() as f64);
    }

    pub fn get_standard_deviation(&self) -> f64 {
        return (self.get_variance() as f64).sqrt();
    }

    pub fn count(&self) -> usize {
        return self.rolling_average.count();
    }
}