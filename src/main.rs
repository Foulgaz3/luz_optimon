mod lunaluz_deserialization;
mod schedules;

use std::{
    collections::HashMap,
    fs,
    sync::{Arc, OnceLock},
};

use axum::{
    routing::{get, post},
    Json, Router,
};

use chrono::{DateTime, Utc};

use lunaluz_deserialization::*;
use schedules::{parse_schedules, Schedule, VarSchedule};
use serde::Serialize;
use serde_json::Value;

type ScheduleMap = Arc<HashMap<String, Schedule<Value>>>;

static SCHEDULES: OnceLock<ScheduleMap> = OnceLock::new();

pub fn schedules() -> ScheduleMap {
    SCHEDULES.get().expect("SCHEDULES not initialized").clone()
}

#[derive(Serialize)]
struct ScheduleResponse {
    time: DateTime<Utc>,
    values: HashMap<String, String>,
}

async fn fetch_variable() -> Json<ScheduleResponse> {
    let time: DateTime<Utc> = Utc::now();
    let binding = schedules();

    let mut response = ScheduleResponse {
        time,
        values: HashMap::new(),
    };

    for var in binding.keys() {
        response.values.insert(
            var.to_string(),
            binding[var].floor_search(&time).to_string(),
        );
    }
    Json(response)
}

#[tokio::main]
async fn main() {
    let json_path = "../example_schedules/example_1.json";
    let json_data = fs::read_to_string(json_path).unwrap();
    let parsed: ScheduleFile = serde_json::from_str(&json_data).unwrap();

    println!("Experiment: {}", parsed.info.experiment_name);
    println!("Variables: {}", parsed.variable_type_specs.len());
    println!("Schedules: {}", parsed.variable_schedules.len());

    let map: ScheduleMap = Arc::new(parse_schedules(parsed.clone()));
    SCHEDULES.set(map).unwrap();

    let app = Router::new().route("/", get(fetch_variable));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
