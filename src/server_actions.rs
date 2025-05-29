use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    lunaluz_deserialization::VariableTypeSpec,
    schedules::{parse_datetime_iso8601, Schedule, VarSchedule},
};

/// Shared, thread-safe map from variable name to its schedule
pub type ScheduleMap = Arc<HashMap<String, Schedule>>;

/// Application state, injected into handlers
#[derive(Clone)]
pub struct AppState {
    pub specs: HashMap<String, VariableTypeSpec>,
    pub schedules: ScheduleMap,
}

/// Query parameters for root endpoint
#[derive(Deserialize)]
pub struct GetVarsParams {
    /// UTC ISO‑8601 timestamp, defaults to now
    pub time: Option<String>,
    /// Include variable types in response; defaults to false
    #[serde(rename = "var_type", alias = "include_types", default)]
    pub include_types: bool,
}

/// Response structure for root endpoint
#[derive(Serialize)]
pub struct ScheduleResponse {
    time: DateTime<Utc>,
    values: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    var_types: Option<HashMap<String, String>>,
}

/// Convert a JSON value to string (unwrapping strings)
pub fn format_json_value(val: Value) -> String {
    if let Value::String(s) = val {
        s
    } else {
        val.to_string()
    }
}

/// Handler to get all variable values at a given time
pub async fn get_vars(
    State(state): State<AppState>,
    Query(params): Query<GetVarsParams>,
) -> Json<ScheduleResponse> {
    // Determine query time
    let time = match params.time {
        Some(t) => parse_datetime_iso8601(&t).unwrap(),
        None => Utc::now(),
    };

    // Collect current values and (optionally) variable types
    let mut values = HashMap::new();
    let mut types = HashMap::new();

    for (var, schedule) in state.schedules.iter() {
        let value = schedule.floor_search(&time);
        values.insert(var.clone(), format_json_value(value));

        if params.include_types {
            types.insert(var.clone(), schedule.var_type());
        }
    }

    let var_types = if params.include_types {
        Some(types)
    } else {
        None
    };

    Json(ScheduleResponse {
        time,
        values,
        var_types,
    })
}

/// Handler to return variable type specs
pub async fn get_specs(State(state): State<AppState>) -> Json<HashMap<String, VariableTypeSpec>> {
    Json(state.specs.clone())
}
