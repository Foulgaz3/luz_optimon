mod lunaluz_deserialization;

use std::fs;

use chrono::{DateTime, TimeDelta, Utc};
use std::time;

use lunaluz_deserialization::*;

fn hours_to_td(hours: Numeric) -> TimeDelta {
    let seconds = f64::from(hours) * 3.6e3;
    let duration = time::Duration::try_from_secs_f64(seconds).unwrap();
    TimeDelta::from_std(duration).unwrap()
}

fn convert_times(times: Vec<Numeric>) -> Vec<TimeDelta> {
    times.into_iter().map(hours_to_td).collect()
}

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

    let start_day: DateTime<Utc> = parsed.info.start_date.parse().unwrap();
    let start_offset: iso8601_duration::Duration = parsed.info.start_offset.parse().unwrap();
    let start_offset = TimeDelta::from_std(start_offset.to_std().unwrap()).unwrap();
    // let period = TimeDelta::hours(schedule.period);
    dbg!(start_day);
    dbg!(start_offset);

    // ! when converting times / schedule, need to assert they are in sorted order wrt time;
    // - Maybe just add this as a part of the specification and add debug check;

    let times = schedule.times.map(convert_times).unwrap();
    let values = schedule.values.unwrap();

    let default_val = parsed.variable_type_specs[&schedule.variable_type]
        .default
        .clone();

    let ref_times = [0, 3, 6, 9, 12, 15, 18, 21, 24];
    for time in ref_times {
        let time = TimeDelta::hours(time);
        let idx = fetch_index(&times, &time);
        let value = if idx == 0 && time < times[0] {
            default_val.clone()
        } else {
            values[idx].clone()
        };
        println!("Time: {time}, Value: {}", value);
    }

    Ok(())
}
