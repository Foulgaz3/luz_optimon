mod lunaluz_deserialization;
mod schedules;
mod server_actions;

use std::{fs, net::{IpAddr, SocketAddr}, sync::Arc};

use axum::{routing::{get, post}, Router};

use lunaluz_deserialization::*;
use schedules::parse_schedules;
use server_actions::{get_specs, get_vars, post_vars, AppState};

#[tokio::main]
async fn main() {
    let json_path = "../example_schedules/example_1.json";
    let json_data = fs::read_to_string(json_path).unwrap();
    let parsed: ScheduleFile = serde_json::from_str(&json_data).unwrap();

    println!("Experiment Name: {}", parsed.info.experiment_name);

    let map = Arc::new(parse_schedules(parsed.clone()));

    let state = AppState {
        specs: parsed.var_type_specs,
        schedules: map
    };

    let app = Router::new()
        .route("/", get(get_vars))
        .route("/specs", get(get_specs))
        .route("/vars", post(post_vars).get(get_vars))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let ip_addr: IpAddr = "127.0.0.1".parse().expect("Couldn't parse ip address");
    let port = 3000;

    let socket = SocketAddr::new(ip_addr, port);
    let listener = tokio::net::TcpListener::bind(socket).await.expect("Failed to create TCP listener");
    println!("Server is listening to {socket}, press Ctrl-C to exit program");
    axum::serve(listener, app).await.unwrap();
}
