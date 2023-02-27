use crate::stats::{MinMax, MinMaxChange, RollingStandardDeviation};
use crate::stats::MinMaxChange::{FirstMinAndMax, NewMax, NewMin};
use crate::stats::totalstandarddeviation::TotalStandardDeviation;

pub struct RollingStats {
    standard_deviation_ration: f64,
    roll_std_dev: RollingStandardDeviation,
    total_std_dev: TotalStandardDeviation,
    roll_av_min_max: MinMax<f64>
}

#[derive(Debug)]
pub struct ValueOfInterest {
    rolling_average_min_max_change: Option<MinMaxChange<f64>>,
    individual_outlier: Option<f64>,
    rolling_average_outlier: Option<f64>
}

impl ValueOfInterest {

    pub fn get_rolling_average_min_max_change(&self) -> &Option<MinMaxChange<f64>> {
        return &self.rolling_average_min_max_change;
    }

    pub fn get_individual_outlier(&self) -> &Option<f64> {
        return &self.individual_outlier;
    }

    pub fn get_rolling_average_outlier(&self) -> &Option<f64> {
        return &self.rolling_average_outlier;
    }
}

impl RollingStats {
    pub fn new(size: usize, standard_deviation_ration: f64) -> Self {
        Self {
            standard_deviation_ration,
            roll_std_dev: RollingStandardDeviation::new(size),
            total_std_dev: TotalStandardDeviation::new(),
            roll_av_min_max: MinMax::NoValues
        }
    }

    pub fn add_value(&mut self, value: f64) -> Option<ValueOfInterest> {
        self.roll_std_dev.add_value(value);
        self.total_std_dev.add_value(value);

        if self.roll_std_dev.is_full() {

            let average = self.total_std_dev.get_average();
            let standard_deviation = self.total_std_dev.get_standard_deviation();
            let rolling_average = self.roll_std_dev.get_average();
            let rolling_standard_deviation = self.roll_std_dev.get_standard_deviation();

            let individual_outlier;
            if (rolling_average - value).abs() > rolling_standard_deviation * self.standard_deviation_ration {
                individual_outlier = Some(value);
            } else {
                individual_outlier = None;
            }

            let rolling_average_outlier;
            if (average - rolling_average).abs() > standard_deviation * self.standard_deviation_ration {
                rolling_average_outlier = Some(value);
            } else {
                rolling_average_outlier = None;
            }

            let rolling_average_min_max_change = match self.roll_av_min_max.add_value(rolling_average) {
                None => None,
                Some(FirstMinAndMax(only_value)) => Some(FirstMinAndMax(*only_value)),
                Some(NewMin(min)) => Some(NewMin(*min)),
                Some(NewMax(max)) => Some(NewMax(*max)),
            };

            if individual_outlier.is_some() ||
                rolling_average_outlier.is_some() ||
                rolling_average_min_max_change.is_some() {

                //error!("{:?}", self.roll_av_min_max);

                return Some(ValueOfInterest {
                    rolling_average_min_max_change,
                    individual_outlier,
                    rolling_average_outlier
                });
            }
        }

        return None;
    }

    pub fn get_rolling_average_min(&self) -> Option<&f64> {
        return self.roll_av_min_max.get_min();
    }

    pub fn get_rolling_average_max(&self) -> Option<&f64> {
        return self.roll_av_min_max.get_max();
    }

    pub fn get_rolling_average(&self) -> f64 {
        return self.roll_std_dev.get_average();
    }

    pub fn get_rolling_standard_deviation(&self) -> f64 {
        return self.roll_std_dev.get_standard_deviation();
    }

    pub fn get_average(&self) -> f64 {
        return self.total_std_dev.get_average();
    }

    pub fn get_standard_deviation(&self) -> f64 {
        return self.total_std_dev.get_standard_deviation();
    }
}