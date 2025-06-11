mod lunaluz_deserialization;
mod schedules;
mod server_actions;

use std::{fs, net::{IpAddr, SocketAddr}, path::PathBuf, sync::Arc};

use axum::{routing::{get, post}, Router};

use clap::Parser;
use lunaluz_deserialization::*;
use schedules::parse_schedules;
use server_actions::{get_specs, get_vars, post_vars, AppState};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// Path to the schedule file
    filename: PathBuf,

    /// IP address to bind to (default: 127.0.0.1)
    #[arg(short, long, default_value = "127.0.0.1")]
    ip: IpAddr,

    /// Port number to bind to (default: 3000)
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let json_data = fs::read_to_string(args.filename).unwrap();
    let parsed: ScheduleFile = serde_json::from_str(&json_data).unwrap();

    println!("Experiment Name: {}", parsed.info.experiment_name);

    let map = parse_schedules(parsed.clone()).unwrap();

    let state = AppState {
        specs: parsed.var_type_specs,
        schedules: Arc::new(map)
    };

    let app = Router::new()
        .route("/", get(get_vars))
        .route("/specs", get(get_specs))
        .route("/vars", post(post_vars).get(get_vars))
        .with_state(state);

    // run the app with hyper
    let socket = SocketAddr::new(args.ip, args.port);
    let listener = tokio::net::TcpListener::bind(socket).await.expect("Failed to create TCP listener");
    println!("Server is listening to {socket}, press Ctrl-C to exit program");
    axum::serve(listener, app).await.unwrap();
}
