mod lunaluz_deserialization;
mod schedules;

use std::{collections::HashMap, fs};

use chrono::{DateTime, TimeDelta, Utc};
use schedules::{hours_to_td, midnight, ConstantSchedule, PeriodicSchedule, VarSchedule};

use lunaluz_deserialization::*;
use serde_json::Value;

fn parse_schedules(file: ScheduleFile) -> HashMap<String, Box<dyn VarSchedule<Value>>> {
    let start_date: DateTime<Utc> = file.info.start_date.parse().unwrap();

    let t24_start_offset: iso8601_duration::Duration = file.info.start_offset.parse().unwrap();
    let t24_start_offset = TimeDelta::from_std(t24_start_offset.to_std().unwrap()).unwrap();

    let t24_start_point: DateTime<Utc> = midnight(&start_date) + t24_start_offset;

    let var_type_specs = file.variable_type_specs;

    let get_default = |var_type| var_type_specs[&var_type].default.clone();

    let mut schedules: HashMap<String, Box<dyn VarSchedule<Value>>> = HashMap::new();
    for (name, schedule) in file.variable_schedules.into_iter() {
        let schedule: Box<dyn VarSchedule<Value>> = match schedule.schedule_type() {
            ScheduleType::Constant | ScheduleType::Default => {
                let value = schedule.value.unwrap_or(get_default(schedule.variable_type));
                Box::new(ConstantSchedule::new(value))
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
                Box::new(PeriodicSchedule::new(
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_path = "../example_schedules/example_1.json";
    let json_data = fs::read_to_string(json_path)?;
    let parsed: ScheduleFile = serde_json::from_str(&json_data)?;

    println!("Experiment: {}", parsed.info.experiment_name);
    println!("Variables: {}", parsed.variable_type_specs.len());
    println!("Schedules: {}", parsed.variable_schedules.len());

    let schedules = parse_schedules(parsed.clone());

    for (name, sched) in parsed.variable_schedules.into_iter() {
        println!(" - {}: {:?}", name, sched.schedule_type());
    }

    dbg!(&parsed.variable_type_specs);

    dbg!(&parsed.info);

    // let schedule = &schedules["red_led.duty_cycle"];
    // dbg!(schedule);

    // ! when converting times / schedule, need to assert they are in sorted order wrt time;
    // - Maybe just add this as a part of the specification and add debug check;

    let ref_time: DateTime<Utc> = "2025-05-23T00:00:00+00:00".parse().unwrap();
    let ref_times = [0, 3, 6, 9, 12, 15, 18, 21, 24];
    let times: Vec<DateTime<Utc>> = ref_times
        .into_iter()
        .map(|v| ref_time + TimeDelta::hours(v))
        .collect();

    for var_name in ["red_led.duty_cycle", "green_led.duty_cycle"] {
        println!("Variable: {var_name}");
        let values = &schedules[var_name].floor_multi_search(&times);
        for (time, value) in times.iter().zip(values) {
            println!("Time: {time}, Value: {}", value);
        }
    }

    Ok(())
}
