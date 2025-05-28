mod lunaluz_deserialization;
mod schedules;
mod server_actions;

use std::{fs, sync::Arc};

use axum::{routing::get, Router};

use lunaluz_deserialization::*;
use schedules::parse_schedules;
use server_actions::{get_vars, set_schedules};

#[tokio::main]
async fn main() {
    let json_path = "../example_schedules/example_1.json";
    let json_data = fs::read_to_string(json_path).unwrap();
    let parsed: ScheduleFile = serde_json::from_str(&json_data).unwrap();

    println!("Experiment: {}", parsed.info.experiment_name);
    println!("Variables: {}", parsed.var_type_specs.len());
    println!("Schedules: {}", parsed.variable_schedules.len());

    let map = Arc::new(parse_schedules(parsed.clone()));
    set_schedules(map).unwrap();

    let app = Router::new().route("/", get(get_vars));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
