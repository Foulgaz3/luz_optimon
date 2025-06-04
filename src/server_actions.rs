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

#[derive(Deserialize)]
pub struct ScheduleQuery {
    time: Option<String>,
    times: Option<Vec<String>>,
    vars: Option<Vec<String>>,
}

pub async fn post_vars(
    State(state): State<AppState>,
    Json(payload): Json<ScheduleQuery>,
) -> Result<Json<Vec<ScheduleResponse>>, &'static str> {
    if payload.time.is_some() && payload.times.is_some() {
        return Err("Bad request; included both time and times");
    }

    let vars: Vec<String> = match payload.vars {
        Some(var_list) => {
            if !var_list.iter().all(|v| state.schedules.contains_key(v)) {
                return Err("Requested one or more unknown variables");
            };
            var_list
        }
        None => state.schedules.keys().map(|v| v.to_string()).collect(),
    };

    let replies = if let Some(times) = payload.times {
        let times: Vec<DateTime<Utc>> = times
            .iter()
            .map(|t| parse_datetime_iso8601(&t).unwrap())
            .collect();

        let mut value_map = vec![HashMap::new(); times.len()];
        for var in vars.into_iter() {
            let schedule = &state.schedules[&var];
            let var_values = schedule.floor_multi_search(&times);
            for (i, value) in var_values.into_iter().enumerate() {
                value_map[i].insert(var.clone(), format_json_value(value));
            }
        }

        times.iter().zip(value_map)
            .map(|(&time, values)| ScheduleResponse {
                time,
                values,
                var_types: None,
            }).collect()
    } else {
        let time = match payload.time {
            Some(t) => parse_datetime_iso8601(&t).unwrap(),
            None => Utc::now(),
        };

        let mut values = HashMap::new();

        for var in vars.iter() {
            let schedule = &state.schedules[var];
            let value = schedule.floor_search(&time);
            values.insert(var.clone(), format_json_value(value));
        }

        vec![ScheduleResponse {
            time,
            values,
            var_types: None,
        }]
    };

    Ok(Json(replies))
}
