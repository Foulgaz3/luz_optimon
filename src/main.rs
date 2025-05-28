mod lunaluz_deserialization;
mod schedules;

use std::{
    collections::HashMap,
    fs,
    sync::{Arc, OnceLock},
};

use axum::{
    extract::Query,
    routing::get,
    Json, Router,
};

use chrono::{DateTime, Utc};

use lunaluz_deserialization::*;
use schedules::{parse_datetime_iso8601, parse_schedules, Schedule, VarSchedule};
use serde::{Deserialize, Serialize};
use serde_json::Value;

type ScheduleMap = Arc<HashMap<String, Schedule<Value>>>;

static SCHEDULES: OnceLock<ScheduleMap> = OnceLock::new();

pub fn schedules() -> ScheduleMap {
    SCHEDULES.get().expect("SCHEDULES not initialized").clone()
}

#[derive(Deserialize)]
struct QueryParams {
    time: Option<String>,
}

#[derive(Serialize)]
struct ScheduleResponse {
    time: DateTime<Utc>,
    values: HashMap<String, String>,
}

fn format_value(val: Value) -> String {
    match val {
        Value::String(_) => val.as_str().unwrap().to_string(),
        _ => val.to_string(),
    }
}

async fn fetch_variable(Query(params): Query<QueryParams>) -> Json<ScheduleResponse> {
    // retrieves all variable values at a given query time
    let binding = schedules();

    let time = match params.time {
        Some(t) => parse_datetime_iso8601(&t).unwrap(),
        None => Utc::now(),
    };

    let mut response = ScheduleResponse {
        time,
        values: HashMap::new(),
    };

    for (var, schedule) in binding.iter() {
        let value = schedule.floor_search(&time);
        let value = format_value(value);
        response.values.insert(var.to_string(), value);
    }

    Json(response)
}

#[tokio::main]
async fn main() {
    let json_path = "../example_schedules/example_1.json";
    let json_data = fs::read_to_string(json_path).unwrap();
    let parsed: ScheduleFile = serde_json::from_str(&json_data).unwrap();

    println!("Experiment: {}", parsed.info.experiment_name);
    println!("Variables: {}", parsed.var_type_specs.len());
    println!("Schedules: {}", parsed.variable_schedules.len());

    let map: ScheduleMap = Arc::new(parse_schedules(parsed.clone()));
    SCHEDULES.set(map).unwrap();

    let app = Router::new().route("/", get(fetch_variable));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
