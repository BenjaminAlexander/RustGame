mod rollingaverage;
mod rollingstandarddeviation;
mod minmax;
mod totalaverage;
mod rollingstats;
mod totalstandarddeviation;

pub use self::minmax::{MinMax, MinMaxChange};
pub use self::rollingaverage::RollingAverage;
pub use self::rollingstandarddeviation::RollingStandardDeviation;
pub use self::rollingstats::{RollingStats, ValueOfInterest};
pub use self::totalaverage::TotalAverage;
pub use self::totalstandarddeviation::TotalStandardDeviation;