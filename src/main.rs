mod lunaluz_deserialization;
mod schedules;

use std::{cell::OnceCell, collections::HashMap, fs, sync::{Arc, OnceLock}};

use chrono::{DateTime, TimeDelta, Utc};

use lunaluz_deserialization::*;
use schedules::{parse_schedules, Schedule, VarSchedule};
use serde_json::Value;

type ScheduleMap = Arc<HashMap<String, Schedule<Value>>>;

static SCHEDULES: OnceLock<ScheduleMap> = OnceLock::new();
    
pub fn schedules() -> ScheduleMap {
    SCHEDULES.get().expect("SCHEDULES not initialized").clone()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_path = "../example_schedules/example_1.json";
    let json_data = fs::read_to_string(json_path)?;
    let parsed: ScheduleFile = serde_json::from_str(&json_data)?;

    println!("Experiment: {}", parsed.info.experiment_name);
    println!("Variables: {}", parsed.variable_type_specs.len());
    println!("Schedules: {}", parsed.variable_schedules.len());

    let map: ScheduleMap = Arc::new(parse_schedules(parsed.clone()));
    SCHEDULES.set(map).unwrap();

    for (name, sched) in parsed.variable_schedules.into_iter() {
        println!(" - {}: {:?}", name, sched.schedule_type());
    }

    dbg!(&parsed.variable_type_specs);

    dbg!(&parsed.info);

    let binding = schedules();
    let schedule = &binding.get("red_led.duty_cycle");
    dbg!(schedule);

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
        let values = binding.get(var_name).unwrap().floor_multi_search(&times);
        for (time, value) in times.iter().zip(values) {
            println!("Time: {time}, Value: {}", value);
        }
    }

    Ok(())
}
