use std::{collections::HashMap, sync::{Arc, OnceLock}};

use axum::{extract::Query, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::schedules::{parse_datetime_iso8601, Schedule, VarSchedule};

pub type ScheduleMap = Arc<HashMap<String, Schedule<Value>>>;

static SCHEDULES: OnceLock<ScheduleMap> = OnceLock::new();

pub fn schedules() -> ScheduleMap {
    SCHEDULES.get().expect("SCHEDULES not initialized").clone()
}

pub fn set_schedules(map: ScheduleMap) -> Result<(), ScheduleMap> {
    SCHEDULES.set(map)
}

#[derive(Deserialize)]
pub struct GetQueryParams {
    time: Option<String>,
    var_type: Option<bool>
}

#[derive(Serialize)]
pub struct ScheduleResponse {
    time: DateTime<Utc>,
    values: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    var_types: Option<HashMap<String, String>>,
}

pub fn format_json_value(val: Value) -> String {
    if let Value::String(s) = val {
        s.clone()
    } else {
        val.to_string()
    }
}

pub async fn get_vars(Query(params): Query<GetQueryParams>) -> Json<ScheduleResponse> {
    // retrieves all variable values at a given query time
    let binding = schedules();

    let time = match params.time {
        Some(t) => parse_datetime_iso8601(&t).unwrap(),
        None => Utc::now(),
    };

    let include_types = params.var_type.unwrap_or(false);

    let mut values = HashMap::new();
    let mut types = HashMap::new();

    for (var, schedule) in binding.iter() {
        let value = format_json_value(schedule.floor_search(&time));
        values.insert(var.clone(), value);

        if include_types {
            types.insert(var.clone(), schedule.var_type());
        }
    }

    let var_types = if include_types { Some(types) } else { None };

    let response = ScheduleResponse {
        time,
        values,
        var_types
    };

    Json(response)
}
