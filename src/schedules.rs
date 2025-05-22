use chrono::{TimeDelta, DateTime, Utc};

pub struct PeriodicSchedule<T> {
    pub start_date: DateTime<Utc>,
    pub start_offset: TimeDelta,
    pub period: TimeDelta,
    pub times: Vec<TimeDelta>,
    pub values: Vec<T>,
    pub default_val: T,
}