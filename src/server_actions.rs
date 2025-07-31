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
    schedules::{parse_datetime_iso8601, NamespaceMap, ScheduleMap, VarSchedule},
};

/// Application state, injected into handlers
#[derive(Clone)]
pub struct AppState {
    pub specs: HashMap<String, VariableTypeSpec>,
    pub schedules: Arc<ScheduleMap>,
    pub ext_schedules: Arc<NamespaceMap>
}

/// Query parameters for root endpoint
#[derive(Deserialize)]
pub struct GetVarsParams {
    /// UTC ISO‑8601 timestamp, defaults to now
    pub time: Option<String>,
    /// Include variable types in response; defaults to false
    #[serde(rename = "var_type", alias = "include_types", default)]
    pub include_types: bool,
    /// Namespace ID (used by extensions with private namespaces)
    pub namespace: Option<String>,
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
    Query(payload): Query<GetVarsParams>,
) -> Result<Json<GetScheduleResponse>, String> {
    // Determine query time
    let time = match payload.time {
        Some(t) => parse_datetime_iso8601(&t)?,
        None => Utc::now(),
    };

    // Collect current values and (optionally) variable types
    let mut values = HashMap::new();
    let mut types = HashMap::new();

    let schedules = match payload.namespace {
        Some(id) => state.ext_schedules.get(&id).ok_or(format!("Unknown Namespace: '{id}'"))?,
        None => &state.schedules,
    };

    for (var, schedule) in schedules.iter() {
        let value = schedule.floor_search(&time);
        values.insert(var.clone(), value);

        if payload.include_types {
            types.insert(var.clone(), schedule.var_type());
        }
    }

    let var_types = if payload.include_types {
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
    /// UTC ISO‑8601 timestamp, defaults to now
    time: Option<String>,
    /// UTC ISO‑8601 timestamps, defaults to now
    times: Option<Vec<String>>,
    /// Names of requested variables
    vars: Option<Vec<String>>,
    /// Namespace ID (defaults to global namespace)
    namespace: Option<String>,
}

// ? Should I add support for single-val returns
// - allow return of "time" and skip serialization if None
// - would need clearly communicated
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

    let schedules = match payload.namespace {
        Some(id) => state.ext_schedules.get(&id).ok_or(format!("Unknown Namespace: '{id}'"))?,
        None => &state.schedules,
    };

    let vars: Vec<String> = match payload.vars {
        Some(var_list) => {
            if !var_list.iter().all(|v| schedules.contains_key(v)) {
                return Err("Requested one or more unknown variables".to_string());
            };
            var_list
        }
        None => schedules.keys().map(|v| v.to_string()).collect(),
    };

    let replies = if let Some(times) = payload.times {
        let times: Result<Vec<DateTime<Utc>>, String> = times
            .iter()
            .map(|t| parse_datetime_iso8601(&t))
            .collect();
        let times = times?;

        let mut values = HashMap::new();
        for var in vars.into_iter() {
            let schedule = &schedules[&var];
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
            let schedule = &schedules[var];
            let value = schedule.floor_search(&time);
            values.insert(var.clone(), vec![value]);
        }

        PostScheduleResponse { times, values }
    };

    Ok(Json(replies))
}
