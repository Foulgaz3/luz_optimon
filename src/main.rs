mod lunaluz_deserialization;

use std::fs;

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

    Ok(())
}
