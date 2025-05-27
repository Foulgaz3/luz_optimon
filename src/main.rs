mod lunaluz_deserialization;
mod schedules;

use std::{collections::HashMap, fs};

use chrono::{DateTime, TimeDelta, Utc};
use schedules::{midnight, ConstantSchedule, PeriodicSchedule, VarSchedule};

use lunaluz_deserialization::*;
use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_path = "../example_schedules/example_1.json";
    let json_data = fs::read_to_string(json_path)?;
    let parsed: ScheduleFile = serde_json::from_str(&json_data)?;

    println!("Experiment: {}", parsed.info.experiment_name);
    println!("Variables: {}", parsed.variable_type_specs.len());
    println!("Schedules: {}", parsed.variable_schedules.len());
 

    let start_date: DateTime<Utc> = parsed.info.start_date.parse().unwrap();

    let t24_start_offset: iso8601_duration::Duration = parsed.info.start_offset.parse().unwrap();
    let t24_start_offset = TimeDelta::from_std(t24_start_offset.to_std().unwrap()).unwrap();
    
    let t24_start_point: DateTime<Utc> = midnight(&start_date) + t24_start_offset;
        

    let mut schedules: HashMap<String, Box<dyn VarSchedule<Value>>> = HashMap::new();

    for (name, sched) in parsed.variable_schedules.into_iter() {
        let sched_type = sched.schedule_type();
        println!(" - {}: {:?}", name, sched_type);
        let schedule: Box<dyn VarSchedule<Value>> = match sched_type {
            ScheduleType::Constant => {
                let value = sched.value.unwrap();
                Box::new(ConstantSchedule::new(value))
            },
            ScheduleType::Default => {
                let default_value = parsed.variable_type_specs[&sched.variable_type].default.clone();
                Box::new(ConstantSchedule::new(default_value))
            },
            ScheduleType::Periodic => {
                let period = sched.period.unwrap();
                let start_point = if f64::from(period) == 24.0 {
                    t24_start_point
                } else {
                    start_date // ! need to add optional offset here
                };

                let times = sched.times.unwrap();
                let values = sched.values.unwrap();
                let default_value = parsed.variable_type_specs[&sched.variable_type].default.clone();
                Box::new(PeriodicSchedule::new(
                    start_point,
                    period,
                    times,
                    values,
                    default_value
                ))
            },
        };
        schedules.insert(name, schedule);
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

    let values = &schedules["red_led.duty_cycle"].floor_multi_search(&times);

    for (time, value) in times.iter().zip(values) {
        println!("Time: {time}, Value: {}", value);
    }

    Ok(())
}
