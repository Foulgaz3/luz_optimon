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
struct GetQueryParams {
    time: Option<String>,
}

#[derive(Serialize)]
struct ScheduleResponse {
    time: DateTime<Utc>,
    values: HashMap<String, String>,
}

fn format_json_value(val: Value) -> String {
    if let Value::String(s) = val {
        s.clone()
    } else {
        val.to_string()
    }
}

async fn get_vars(Query(params): Query<GetQueryParams>) -> Json<ScheduleResponse> {
    // retrieves all variable values at a given query time
    let binding = schedules();

    let time = match params.time {
        Some(t) => parse_datetime_iso8601(&t).unwrap(),
        None => Utc::now(),
    };

    let mut values = HashMap::new();
    for (var, schedule) in binding.iter() {
        let value = format_json_value(schedule.floor_search(&time));
        values.insert(var.clone(), value);
    }

    let response = ScheduleResponse {
        time,
        values,
    };

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

    let app = Router::new().route("/", get(get_vars));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
