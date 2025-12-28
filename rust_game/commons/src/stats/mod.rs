mod minmax;
mod rollingaverage;
mod rollingstandarddeviation;
mod rollingstats;
mod rollingstatslogger;
mod totalaverage;
mod totalstandarddeviation;

pub use self::{
    minmax::{
        MinMax,
        MinMaxChange,
    },
    rollingaverage::RollingAverage,
    rollingstandarddeviation::RollingStandardDeviation,
    rollingstats::{
        RollingStats,
        ValueOfInterest,
    },
    rollingstatslogger::RollingStatsLogger,
    totalaverage::TotalAverage,
    totalstandarddeviation::TotalStandardDeviation,
};
