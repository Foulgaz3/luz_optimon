mod lunaluz_deserialization;
mod schedules;
mod server_actions;

use std::{collections::HashMap, fs, sync::Arc};

use axum::{extract::State, routing::get, Json, Router};

use lunaluz_deserialization::*;
use schedules::parse_schedules;
use server_actions::{get_vars, set_schedules};

#[derive(Clone)]
struct AppState {
    specs: HashMap<String, VariableTypeSpec>,
}

async fn get_specs(State(state): State<AppState>) -> Json<HashMap<String, VariableTypeSpec>> {
    Json(state.specs.clone())
}

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

    let state = AppState {
        specs: parsed.var_type_specs,
    };

    let app = Router::new()
        .route("/", get(get_vars))
        .route("/specs", get(get_specs))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
