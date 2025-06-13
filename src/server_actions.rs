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
    schedules::{parse_datetime_iso8601, ScheduleMap, VarSchedule},
};

/// Application state, injected into handlers
#[derive(Clone)]
pub struct AppState {
    pub specs: HashMap<String, VariableTypeSpec>,
    pub schedules: Arc<ScheduleMap>,
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
pub struct GetScheduleResponse {
    time: DateTime<Utc>,
    values: HashMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    var_types: Option<HashMap<String, String>>,
}

/// Handler to get all variable values at a given time
pub async fn get_vars(
    State(state): State<AppState>,
    Query(params): Query<GetVarsParams>,
) -> Result<Json<GetScheduleResponse>, String> {
    // Determine query time
    let time = match params.time {
        Some(t) => parse_datetime_iso8601(&t)?,
        None => Utc::now(),
    };

    // Collect current values and (optionally) variable types
    let mut values = HashMap::new();
    let mut types = HashMap::new();

    for (var, schedule) in state.schedules.iter() {
        let value = schedule.floor_search(&time);
        values.insert(var.clone(), value);

        if params.include_types {
            types.insert(var.clone(), schedule.var_type());
        }
    }

    let var_types = if params.include_types {
        Some(types)
    } else {
        None
    };

    Ok(Json(GetScheduleResponse {
        time,
        values,
        var_types,
    }))
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

#[derive(Serialize, Deserialize)]
pub struct PostScheduleResponse {
    times: Vec<DateTime<Utc>>,
    values: HashMap<String, Vec<Value>>,
}

pub async fn post_vars(
    State(state): State<AppState>,
    Json(payload): Json<ScheduleQuery>,
) -> Result<Json<PostScheduleResponse>, String> {
    if payload.time.is_some() && payload.times.is_some() {
        return Err("Bad request; included both time and times".to_string());
    }

    let vars: Vec<String> = match payload.vars {
        Some(var_list) => {
            if !var_list.iter().all(|v| state.schedules.contains_key(v)) {
                return Err("Requested one or more unknown variables".to_string());
            };
            var_list
        }
        None => state.schedules.keys().map(|v| v.to_string()).collect(),
    };

    let replies = if let Some(times) = payload.times {
        let times: Result<Vec<DateTime<Utc>>, String> = times
            .iter()
            .map(|t| parse_datetime_iso8601(&t))
            .collect();
        let times = times?;

        let mut values = HashMap::new();
        for var in vars.into_iter() {
            let schedule = &state.schedules[&var];
            let var_values = schedule.floor_multi_search(&times);
            values.insert(var, var_values);
        }
        PostScheduleResponse { times, values }
    } else {
        let time = match payload.time {
            Some(t) => parse_datetime_iso8601(&t)?,
            None => Utc::now(),
        };

        let times = vec![time];
        let mut values = HashMap::new();

        for var in vars.iter() {
            let schedule = &state.schedules[var];
            let value = schedule.floor_search(&time);
            values.insert(var.clone(), vec![value]);
        }

        PostScheduleResponse { times, values }
    };

    Ok(Json(replies))
}
