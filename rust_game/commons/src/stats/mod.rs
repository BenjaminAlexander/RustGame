mod rollingaverage;
mod rollingstandarddeviation;
mod minmax;

pub use self::minmax::{MinMax, MinMaxChange};
pub use self::rollingaverage::RollingAverage;
pub use self::rollingstandarddeviation::RollingStandardDeviation;