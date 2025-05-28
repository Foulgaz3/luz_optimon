use std::collections::HashMap;

use chrono::{DateTime, Datelike, TimeDelta, TimeZone, Utc, NaiveDateTime};
use serde_json::Value;

use crate::lunaluz_deserialization::{Numeric, ScheduleFile, ScheduleType};

pub fn midnight(time: &DateTime<Utc>) -> DateTime<Utc> {
    // retrieve datetime for very start of a given day
    time.timezone()
        .with_ymd_and_hms(time.year(), time.month(), time.day(), 0, 0, 0)
        .unwrap()
}

pub fn parse_datetime_iso8601(input: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    // Attempt RFC 3339 / ISO 8601 extended first
    let result = DateTime::parse_from_rfc3339(input).map(|dt| dt.with_timezone(&Utc));
    if result.is_ok() {
        return result
    }

    // Fallback to known alternative ISO 8601-compatible patterns
    const FORMATS: &[&str] = &[
        "%Y-%m-%dT%H%M%S",     // basic with dashes
        "%Y-%m-%dT%H:%M:%S",   // extended
        "%Y%m%dT%H%M%S",       // compact basic
    ];

    for format in FORMATS {
        if let Ok(naive) = NaiveDateTime::parse_from_str(input, format) {
            return Ok(Utc.from_utc_datetime(&naive));
        }
    }

    // If all formats fail, return the last error from RFC3339 attempt
    result
}

pub fn hours_to_td(hours: Numeric) -> TimeDelta {
    let seconds = f64::from(hours) * 3.6e3;
    let duration = std::time::Duration::try_from_secs_f64(seconds).unwrap();
    TimeDelta::from_std(duration).unwrap()
}

pub fn convert_times(times: Vec<Numeric>) -> Vec<TimeDelta> {
    times.into_iter().map(hours_to_td).collect()
}

pub trait VarSchedule<T> {
    fn floor_search(&self, time: &DateTime<Utc>) -> T;

    fn floor_multi_search(&self, times: &[DateTime<Utc>]) -> Vec<T> {
        times.iter().map(|t| self.floor_search(t)).collect()
    }
}

#[derive(Debug)]
pub enum Schedule<Value> {
    Constant(ConstantSchedule<Value>),
    Periodic(PeriodicSchedule<Value>),
}

impl<T: Clone> VarSchedule<T> for Schedule<T> {
    fn floor_search(&self, time: &DateTime<Utc>) -> T {
        match self {
            Schedule::Constant(c) => c.floor_search(time),
            Schedule::Periodic(p) => p.floor_search(time),
        }
    }

    fn floor_multi_search(&self, times: &[DateTime<Utc>]) -> Vec<T> {
        match self {
            Schedule::Constant(c) => c.floor_multi_search(times),
            Schedule::Periodic(p) => p.floor_multi_search(times),
        }
    }
}

#[derive(Debug)]
pub struct ConstantSchedule<T> {
    pub value: T,
}

impl<T> ConstantSchedule<T>
where
    T: Clone,
{
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T> VarSchedule<T> for ConstantSchedule<T>
where
    T: Clone,
{
    fn floor_search(&self, _time: &DateTime<Utc>) -> T {
        self.value.clone()
    }

    fn floor_multi_search(&self, times: &[DateTime<Utc>]) -> Vec<T> {
        vec![self.value.clone(); times.len()]
    }
}

#[derive(Debug)]
pub struct PeriodicSchedule<T> {
    pub start_point: DateTime<Utc>,
    pub period: TimeDelta,
    pub times: Vec<TimeDelta>,
    pub values: Vec<T>,
    pub default_val: T,
}

impl<T> PeriodicSchedule<T>
where
    T: Clone,
{
    pub fn new(
        start_date: DateTime<Utc>,
        period: Numeric,
        times: Vec<Numeric>,
        values: Vec<T>,
        default_val: T,
    ) -> Self {
        let period = hours_to_td(period);
        let times = convert_times(times);
        Self {
            start_point: start_date,
            period,
            times,
            values,
            default_val,
        }
    }

    pub fn most_recent_start(&self, time: &DateTime<Utc>) -> DateTime<Utc> {
        let elapsed = *time - self.start_point;
        let approx_n = elapsed.num_seconds() / self.period.num_seconds();
        let most_recent_start = self.start_point + self.period * approx_n as i32;
        // ! May need to add / subtract by a period until
        //   most_recent_start is the maximum solution to S = start + k*period
        //   where S still comes before theoretic datetime
        debug_assert!(most_recent_start <= *time);
        most_recent_start
    }

    pub fn fetch_schedule_point(&self, time: &DateTime<Utc>) -> TimeDelta {
        let most_recent_start = self.most_recent_start(time);
        let schedule_time = *time - most_recent_start;
        debug_assert!(schedule_time < self.period);
        schedule_time
    }
}

impl<T> VarSchedule<T> for PeriodicSchedule<T>
where
    T: Clone,
{
    fn floor_search(&self, time: &DateTime<Utc>) -> T {
        let schedule_time = self.fetch_schedule_point(time);
        match self.times.binary_search(&schedule_time) {
            Ok(index) => self.values[index].clone(),
            Err(index) => {
                if index == 0 && schedule_time < self.times[0] {
                    self.default_val.clone()
                } else {
                    self.values[index - 1].clone()
                }
            }
        }
    }
}


pub fn parse_schedules(file: ScheduleFile) -> HashMap<String, Schedule<Value>> {
    let start_date: DateTime<Utc> = file.info.start_date.parse().unwrap();

    let t24_start_offset: iso8601_duration::Duration = file.info.start_offset.parse().unwrap();
    let t24_start_offset = TimeDelta::from_std(t24_start_offset.to_std().unwrap()).unwrap();

    let t24_start_point: DateTime<Utc> = midnight(&start_date) + t24_start_offset;

    let var_type_specs = file.var_type_specs;

    let get_default = |var_type| var_type_specs[&var_type].default.clone();

    let mut schedules: HashMap<String, Schedule<Value>> = HashMap::new();
    for (name, schedule) in file.variable_schedules.into_iter() {
        let schedule: Schedule<Value> = match schedule.schedule_type() {
            ScheduleType::Constant | ScheduleType::Default => {
                let value = schedule.value.unwrap_or(get_default(schedule.variable_type));
                Schedule::Constant(ConstantSchedule::new(value))
            }
            ScheduleType::Periodic => {
                let period = schedule.period.unwrap();

                // if t24, start time = midnight + start offset
                // else, start time = exact timestamp of start
                // + optional time offset in hours
                let start_point = if f64::from(period) == 24.0 {
                    t24_start_point
                } else if let Some(offset_time) = schedule.offset_time {
                    start_date + hours_to_td(offset_time)
                } else {
                    start_date
                };

                let times = schedule.times.unwrap();
                let values = schedule.values.unwrap();
                let default_value = get_default(schedule.variable_type);
                Schedule::Periodic(PeriodicSchedule::new(
                    start_point,
                    period,
                    times,
                    values,
                    default_value,
                ))
            }
        };
        schedules.insert(name, schedule);
    }

    schedules
}