use lunaluz_deserialization::ScheduleType;
use chrono::{TimeDelta, DateTime};

pub struct PeriodicSchedule<T> {
    pub start_date: DateTime,
    pub start_offset: TimeDelta,
    pub period: TimeDelta,
    pub times: Vec<TimeDelta>,
    pub values: Vec<T>,
}