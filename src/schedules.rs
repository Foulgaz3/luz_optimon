use std::default;

use chrono::{TimeDelta, DateTime, Utc};
use serde_json::Value;

use crate::lunaluz_deserialization::Numeric;

pub fn hours_to_td(hours: Numeric) -> TimeDelta {
    let seconds = f64::from(hours) * 3.6e3;
    let duration = std::time::Duration::try_from_secs_f64(seconds).unwrap();
    TimeDelta::from_std(duration).unwrap()
}

pub fn convert_times(times: Vec<Numeric>) -> Vec<TimeDelta> {
    times.into_iter().map(hours_to_td).collect()
}

// ! Should be able to remove start_offset from here and turn it into a construction thing

pub struct PeriodicSchedule<T> {
    pub start_point: DateTime<Utc>,
    pub period: TimeDelta,
    pub times: Vec<TimeDelta>,
    pub values: Vec<T>,
    pub default_val: T,
}

impl<T> PeriodicSchedule<T> {
    pub fn new(start_date: DateTime<Utc>, period: Numeric, times: Vec<Numeric>, values: Vec<T>, default_val: T) -> Self {
        let period = hours_to_td(period);
        let times = convert_times(times);
        Self {
            start_point: start_date,
            period,
            times,
            values,
            default_val
        }
    }
}