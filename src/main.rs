mod lunaluz_deserialization;
mod schedules;

use std::fs;

use chrono::{DateTime, TimeDelta, Utc};
use schedules::{midnight, PeriodicSchedule, VarSchedule};

use lunaluz_deserialization::*;



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_path = "../example_schedules/example_1.json";
    let json_data = fs::read_to_string(json_path)?;
    let parsed: ScheduleFile = serde_json::from_str(&json_data)?;

    println!("Experiment: {}", parsed.info.experiment_name);
    println!("Variables: {}", parsed.variable_type_specs.len());
    println!("Schedules: {}", parsed.variable_schedules.len());

    for (name, sched) in parsed.variable_schedules.iter() {
        println!(" - {}: {:?}", name, sched.schedule_type());
    }

    dbg!(&parsed.info);

    let schedule = parsed.variable_schedules["red_led.duty_cycle"].clone();
    dbg!(&schedule);

    let start_date: DateTime<Utc> = parsed.info.start_date.parse().unwrap();
    let start_date: DateTime<Utc> = midnight(&start_date);
    let start_offset: iso8601_duration::Duration = parsed.info.start_offset.parse().unwrap();
    let start_offset = TimeDelta::from_std(start_offset.to_std().unwrap()).unwrap();

    // ! when converting times / schedule, need to assert they are in sorted order wrt time;
    // - Maybe just add this as a part of the specification and add debug check;

    let default_val = parsed.variable_type_specs[&schedule.variable_type]
        .default
        .clone();

    let periodic = PeriodicSchedule::new(
        start_date + start_offset,
        schedule.period.unwrap(),
        schedule.times.unwrap(),
        schedule.values.unwrap(),
        default_val,
    );

    let start_point = periodic.start_point.clone();
    let theoretic_day: DateTime<Utc> = "2025-05-23T00:00:00+00:00".parse().unwrap();

    let most_recent_start = periodic.most_recent_start(&theoretic_day);

    let schedule_time = theoretic_day - most_recent_start;
    debug_assert!(schedule_time < periodic.period);

    println!("Start: {start_point}, Current: {theoretic_day}");
    dbg!(periodic.period.num_hours());
    dbg!(most_recent_start);

    let ref_times = [0, 3, 6, 9, 12, 15, 18, 21, 24];
    for time in ref_times {
        let time = theoretic_day + TimeDelta::hours(time);
        let value = periodic.floor_search(&time);
        println!("Time: {time}, Value: {}", value);
    }

    println!("Version 2: Multi-Search");
    let ref_times = [0, 3, 6, 9, 12, 15, 18, 21, 24];
    let times: Vec<DateTime<Utc>> = ref_times.into_iter().map(|v| theoretic_day + TimeDelta::hours(v)).collect();
    
    let values = periodic.floor_multi_search(&times);
    for (time, value) in times.iter().zip(values) {
        println!("Time: {time}, Value: {}", value);
    }

    Ok(())
}
