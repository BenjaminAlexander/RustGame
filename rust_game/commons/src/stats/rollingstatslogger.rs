use std::fmt::Write;
use std::marker::PhantomData;
use log::info;
use crate::stats::RollingStats;
use crate::time::{TimeDuration, TimeValue};

pub struct RollingStatsLogger<T: Into<f64>> {
    min_log_interval: TimeDuration,
    last_log: TimeValue,
    need_to_log: bool,
    rolling_stats: RollingStats,
    phantom: PhantomData<T>
}

impl<T: Into<f64>> RollingStatsLogger<T> {

    pub fn new(size: usize, standard_deviation_ration: f64, min_log_interval: TimeDuration) -> Self {
        Self {
            min_log_interval,
            last_log: TimeValue::now(),
            need_to_log: false,
            rolling_stats: RollingStats::new(size, standard_deviation_ration),
            phantom: PhantomData::default()
        }
    }

    pub fn add_value(&mut self, value: T) {
        let now = TimeValue::now();

        if let Some(value_of_interest) = self.rolling_stats.add_value(value.into()) {
            if now.duration_since(&self.last_log) > self.min_log_interval {

                // log now
                let mut string = String::new();

                if let Some(individual_outlier) = value_of_interest.get_individual_outlier() {
                    writeln!(string, "Individual Outlier: {}", *individual_outlier).unwrap();
                }

                if let Some(rolling_average_outlier) = value_of_interest.get_rolling_average_outlier() {
                    writeln!(string, "Rolling Average Outlier: {}", *rolling_average_outlier).unwrap();
                }

                if let Some(rolling_average_min_max_change) = value_of_interest.get_rolling_average_min_max_change() {
                    writeln!(string, "Rolling Average Min/Max Change: {}", *rolling_average_min_max_change).unwrap();
                }

                self.get_stats_as_string(&mut string);

                info!("{}", string);

                self.last_log = now;
                self.need_to_log = false;

            } else {
                self.need_to_log = true;
            }
        } else if self.need_to_log && now.duration_since(&self.last_log) > self.min_log_interval {

            let mut string = String::new();

            self.get_stats_as_string(&mut string);

            info!("{}", string);

            self.last_log = now;
            self.need_to_log = false;
        }
    }

    fn get_stats_as_string(&self, string: &mut String) {

        let rolling_average_min = match self.rolling_stats.get_rolling_average_min() {
            Some(f) => format!("{}", f),
            None => String::new()
        };

        let rolling_average_max = match self.rolling_stats.get_rolling_average_max() {
            Some(f) => format!("{}", f),
            None => String::new()
        };

        writeln!(string, "Rolling Average: {}\n\
            Rolling Standard Deviation: {}\n\
            Rolling Average Min: {}\n\
            Rolling Average Max: {}\n\
            Average: {}\n\
            Standard Deviation: {}",
            self.rolling_stats.get_rolling_average(),
            self.rolling_stats.get_rolling_standard_deviation(),
            rolling_average_min,
            rolling_average_max,
            self.rolling_stats.get_average(),
            self.rolling_stats.get_standard_deviation()
        ).unwrap();
    }
}