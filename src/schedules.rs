use std::collections::HashMap;

use chrono::{DateTime, Datelike, NaiveDateTime, TimeDelta, TimeZone, Utc};
use enum_dispatch::enum_dispatch;
use serde_json::Value;

use crate::lunaluz_deserialization::{ScheduleFile, ScheduleType};

pub fn midnight(time: &DateTime<Utc>) -> DateTime<Utc> {
    // retrieve datetime for very start of a given day
    time.timezone()
        .with_ymd_and_hms(time.year(), time.month(), time.day(), 0, 0, 0)
        .unwrap() // should never happen since derived from &Datetime<Utc>
}

pub fn parse_datetime_iso8601(input: &str) -> Result<DateTime<Utc>, String> {
    // Attempt RFC 3339 / ISO 8601 extended first
    let result = DateTime::parse_from_rfc3339(input)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| format!("Error parsing time: {e}"));
    if result.is_ok() {
        return result;
    }

    // Fallback to known alternative ISO 8601-compatible patterns
    const FORMATS: &[&str] = &[
        "%Y-%m-%dT%H%M%S",   // basic with dashes
        "%Y-%m-%dT%H:%M:%S", // extended
        "%Y%m%dT%H%M%S",     // compact basic
    ];

    for format in FORMATS {
        if let Ok(naive) = NaiveDateTime::parse_from_str(input, format) {
            return Ok(Utc.from_utc_datetime(&naive));
        }
    }

    // If all formats fail, return the last error from RFC3339 attempt
    result
}

fn parse_duration_iso8601(dur: &str) -> Result<TimeDelta, String> {
    let raw_duration = dur
        .parse::<iso8601_duration::Duration>()
        .map_err(|e| format!("Invalid time duration: {e:?}"))?;

    let std_duration = raw_duration
        .to_std()
        .ok_or_else(|| ("Duration contains unsupported units (e.g. months, years)".to_string()))?;

    TimeDelta::from_std(std_duration)
        .map_err(|e| format!("Failed to convert std Duration to TimeDelta: {e}"))
}

pub fn hours_to_td(hours: f64) -> Result<TimeDelta, String> {
    let seconds = hours * 3.6e3;
    let duration = std::time::Duration::try_from_secs_f64(seconds).map_err(|e| e.to_string())?;
    TimeDelta::from_std(duration).map_err(|e| e.to_string())
}

pub fn convert_times(times: Vec<f64>) -> Result<Vec<TimeDelta>, String> {
    times.into_iter().map(hours_to_td).collect()
}
#[enum_dispatch(Schedule)]
pub trait VarSchedule {
    fn var_type(&self) -> String;
    fn floor_search(&self, time: &DateTime<Utc>) -> Value;

    fn floor_multi_search(&self, times: &[DateTime<Utc>]) -> Vec<Value> {
        times.iter().map(|t| self.floor_search(t)).collect()
    }
}

// ! TODO: add tests for each of these both before and after start/end, etc.

#[derive(Debug)]
#[enum_dispatch]
pub enum Schedule {
    Constant(ConstantSchedule),
    Periodic(PeriodicSchedule),
}

#[derive(Debug)]
pub struct ConstantSchedule {
    pub var_type: String,
    pub value: Value,
}

impl ConstantSchedule {
    pub fn new(var_type: String, value: Value) -> Self {
        Self { var_type, value }
    }
}

impl VarSchedule for ConstantSchedule {
    fn var_type(&self) -> String {
        self.var_type.to_owned()
    }
    fn floor_search(&self, _time: &DateTime<Utc>) -> Value {
        self.value.clone()
    }

    fn floor_multi_search(&self, times: &[DateTime<Utc>]) -> Vec<Value> {
        vec![self.value.clone(); times.len()]
    }
}

#[derive(Debug)]
pub struct PeriodicSchedule {
    pub var_type: String,
    pub start_point: DateTime<Utc>,
    pub period: TimeDelta,
    pub times: Vec<TimeDelta>,
    pub values: Vec<Value>,
    pub default_val: Value,
}

impl PeriodicSchedule {
    pub fn new(
        var_type: String,
        start_date: DateTime<Utc>,
        period: f64,
        times: Vec<f64>,
        values: Vec<Value>,
        default_val: Value,
    ) -> Result<Self, String> {
        let period = hours_to_td(period)
            .map_err(|e| format!("Failed to parse period for periodic schedule: {}", e))?;
        let times = convert_times(times)
            .map_err(|e| format!("Failed to parse time(s) for periodic schedule: {}", e))?;
        Ok(Self {
            var_type,
            start_point: start_date,
            period,
            times,
            values,
            default_val,
        })
    }

    pub fn most_recent_start(&self, time: &DateTime<Utc>) -> DateTime<Utc> {
        let elapsed = *time - self.start_point;
        let approx_n = elapsed.num_seconds() / self.period.num_seconds();
        let most_recent_start = self.start_point + self.period * approx_n as i32;
        // ! May need to add / subtract by a period until
        //   most_recent_start is the maximum solution to S = start + k*period
        //   where S still comes before theoretic datetime
        debug_assert!(most_recent_start <= *time); // can fail if fetching time before start
        most_recent_start
    }

    pub fn fetch_schedule_point(&self, time: &DateTime<Utc>) -> TimeDelta {
        let most_recent_start = self.most_recent_start(time);
        let schedule_time = *time - most_recent_start;
        debug_assert!(schedule_time < self.period);
        schedule_time
    }
}

impl VarSchedule for PeriodicSchedule {
    fn var_type(&self) -> String {
        self.var_type.to_owned()
    }

    fn floor_search(&self, time: &DateTime<Utc>) -> Value {
        // todo: add upper bound here too, if provided
        if *time > self.start_point {
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
        } else {
            self.default_val.clone()
        }
    }
}

/// Map from variable name to its schedule
pub type ScheduleMap = HashMap<String, Schedule>;

pub fn parse_schedules(file: ScheduleFile) -> Result<ScheduleMap, String> {
    let start_date = parse_datetime_iso8601(&file.info.start_date)
        .map_err(|e| format!("Invalid start date format: {e}"))?;

    // timezone included to ensure T24 schedules start on the expected day
    // even in periods when UTC time is on a different day than local time
    let timezone = TimeDelta::hours(file.info.timezone);

    let start_offset = parse_duration_iso8601(&file.info.start_offset)?;

    let t24_start_point = start_date + timezone;
    let t24_start_point = midnight(&t24_start_point) + start_offset - timezone;

    let mut schedules: ScheduleMap = HashMap::new();
    for (name, schedule) in file.variable_schedules.into_iter() {
        let spec = file
            .var_type_specs
            .get(&schedule.header.variable_type)
            .ok_or_else(|| format!("Unknown variable type for {name}"))?;

        let schedule: Schedule = match schedule.schedule_type() {
            ScheduleType::Constant | ScheduleType::Default => {
                let value = schedule.value.unwrap_or(spec.default.clone());
                Schedule::Constant(ConstantSchedule::new(schedule.header.variable_type, value))
            }
            ScheduleType::Periodic => {
                let period = schedule
                    .period
                    .ok_or_else(|| format!("No period provided for {name}"))?;

                let start_point = if f64::from(period) == 24.0 {
                    t24_start_point
                } else if let Some(offset_time) = schedule.offset_time {
                    start_date
                        + hours_to_td(offset_time).map_err(|e| {
                            format!("Failed to parse offset time for '{name}': {}", e)
                        })?
                } else {
                    start_date
                };

                let times = schedule
                    .times
                    .ok_or_else(|| format!("No times found for '{name}'"))?;
                let values = schedule
                    .values
                    .ok_or_else(|| format!("No values found for '{name}'"))?;
                let default_value = spec.default.clone();

                Schedule::Periodic(PeriodicSchedule::new(
                    schedule.header.variable_type,
                    start_point,
                    period,
                    times,
                    values,
                    default_value,
                )?)
            }
        };
        schedules.insert(name, schedule);
    }

    Ok(schedules)
}
