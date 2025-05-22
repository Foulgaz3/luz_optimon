mod lunaluz_deserialization;
mod schedules;

use std::fs;

use chrono::{DateTime, TimeDelta, Utc};
use schedules::PeriodicSchedule;

use lunaluz_deserialization::*;

fn fetch_index(times: &[TimeDelta], time: &TimeDelta) -> usize {
    match times.binary_search(time) {
        Ok(index) => index,
        Err(index) => {
            if index == 0 {
                index
            } else {
                index - 1
            }
        }
    }
}

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
    let start_offset: iso8601_duration::Duration = parsed.info.start_offset.parse().unwrap();
    let start_offset = TimeDelta::from_std(start_offset.to_std().unwrap()).unwrap();
    dbg!(start_date);
    dbg!(start_offset);

    // ! when converting times / schedule, need to assert they are in sorted order wrt time;
    // - Maybe just add this as a part of the specification and add debug check;

    let default_val = parsed.variable_type_specs[&schedule.variable_type]
        .default
        .clone();

    let periodic = PeriodicSchedule::new(
        start_date,
        start_offset,
        schedule.period.unwrap(),
        schedule.times.unwrap(),
        schedule.values.unwrap(),
        default_val,
    );

    let start_offset = periodic.start_offset.clone();
    let start_point = periodic.start_date.clone() + start_offset;
    let theoretic_day: DateTime<Utc> = "2025-05-23T19:36:56+00:00".parse().unwrap();

    let elapsed = theoretic_day - (start_point + start_offset);
    let approx_n = elapsed.num_seconds() / periodic.period.num_seconds();
    let most_recent_start = start_point + periodic.period * approx_n as i32;
    // ! May need to add / subtract by a period until
    //   most_recent_start is the maximum solution to S = start + k*period
    //   where S still comes before theoretic datetime
    debug_assert!(most_recent_start < theoretic_day);
    let schedule_time = theoretic_day - most_recent_start;
    debug_assert!(schedule_time < periodic.period);

    println!("Start: {start_point}, Current: {theoretic_day}");
    dbg!(elapsed.num_hours());
    dbg!(approx_n);
    dbg!(periodic.period.num_hours());
    dbg!(most_recent_start);

    // let ref_times = [0, 3, 6, 9, 12, 15, 18, 21, 24];
    // for time in ref_times {
    //     let time = TimeDelta::hours(time);
    //     let idx = fetch_index(&periodic.times, &time);
    //     let value = if idx == 0 && time < periodic.times[0] {
    //         periodic.default_val.clone()
    //     } else {
    //         periodic.values[idx].clone()
    //     };
    //     println!("Time: {time}, Value: {}", value);
    // }

    Ok(())
}
