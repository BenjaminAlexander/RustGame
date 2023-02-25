use crate::stats::{MinMax, MinMaxChange, RollingStandardDeviation};
use crate::stats::MinMaxChange::{FirstMinAndMax, NewMax, NewMin};
use crate::stats::totalstandarddeviation::TotalStandardDeviation;

pub struct RollingStats {
    standard_deviation_ration: f64,
    roll_std_dev: RollingStandardDeviation,
    total_std_dev_of_roll_av: TotalStandardDeviation,
    roll_av_min_max: MinMax<f64>
}

#[derive(Debug)]
pub struct ValueOfInterest {
    rolling_average_min_max_change: Option<MinMaxChange<f64>>,
    individual_outlier: Option<f64>,
    rolling_average_outlier: Option<f64>,
    rolling_average_min: f64,
    rolling_average_max: f64,
    current_rolling_average: f64,
    current_rolling_standard_deviation: f64,
    average_of_rolling_averages: f64,
    standard_deviation_of_rolling_averages: f64,
}

impl RollingStats {

    pub fn new(size: usize, standard_deviation_ration: f64) -> Self {
        Self {
            standard_deviation_ration,
            roll_std_dev: RollingStandardDeviation::new(size),
            total_std_dev_of_roll_av: TotalStandardDeviation::new(),
            roll_av_min_max: MinMax::NoValues
        }
    }

    pub fn add_value(&mut self, value: f64) -> Option<ValueOfInterest> {
        self.roll_std_dev.add_value(value);
        let rolling_average = self.roll_std_dev.get_average();

        if self.roll_std_dev.is_full() {

            let current_rolling_average = self.roll_std_dev.get_average();
            let current_rolling_standard_deviation = self.roll_std_dev.get_standard_deviation();

            let individual_outlier;
            if (current_rolling_average - value).abs() > current_rolling_standard_deviation * self.standard_deviation_ration {
                individual_outlier = Some(value);
            } else {
                individual_outlier = None;
            }

            let average_of_rolling_averages = self.total_std_dev_of_roll_av.get_average();
            let standard_deviation_of_rolling_averages = self.total_std_dev_of_roll_av.get_standard_deviation();

            let rolling_average_outlier;
            if (average_of_rolling_averages - current_rolling_average).abs() > standard_deviation_of_rolling_averages * self.standard_deviation_ration {
                rolling_average_outlier = Some(value);
            } else {
                rolling_average_outlier = None;
            }

            self.total_std_dev_of_roll_av.add_value(rolling_average);

            let rolling_average_min_max_change = match self.roll_av_min_max.add_value(rolling_average) {
                None => None,
                Some(FirstMinAndMax(only_value)) => Some(FirstMinAndMax(*only_value)),
                Some(NewMin(min)) => Some(NewMin(*min)),
                Some(NewMax(max)) => Some(NewMax(*max)),
            };

            if individual_outlier.is_some() ||
                rolling_average_outlier.is_some() ||
                rolling_average_min_max_change.is_some() {

                return Some(ValueOfInterest{
                    rolling_average_min_max_change,
                    individual_outlier,
                    rolling_average_outlier,
                    rolling_average_min: *self.roll_av_min_max.get_min().unwrap(),
                    rolling_average_max: *self.roll_av_min_max.get_max().unwrap(),
                    current_rolling_average,
                    current_rolling_standard_deviation,
                    average_of_rolling_averages,
                    standard_deviation_of_rolling_averages,
                });
            }
        }

        return None;
    }
}