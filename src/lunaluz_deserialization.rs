#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

// ------------------------- Variable Type Spec -------------------------

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
pub enum VarDataType {
    Interval,
    Ratio,
    Nominal,
    Ordinal,
    Administrative,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VariableTypeSpec {
    #[serde(rename = "VariableType")]
    pub var_type: VarDataType,

    #[serde(rename = "DefaultValue")]
    pub default: JsonValue,

    #[serde(rename = "Description")]
    pub description: String,

    #[serde(rename = "Categories", default)]
    pub categories: Option<Vec<String>>,
}

// ------------------------- Schedule Section -------------------------

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScheduleType {
    Constant,
    Periodic,
    Default,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(untagged)]
pub enum Numeric {
    Int(i64),
    Float(f64),
}
impl From<Numeric> for f64 {
    fn from(n: Numeric) -> f64 {
        match n {
            Numeric::Int(i) => i as f64,
            Numeric::Float(f) => f,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ScheduleEntry {
    #[serde(rename = "VariableType")]
    pub variable_type: String,

    #[serde(rename = "ScheduleType", default)]
    schedule_type: Option<ScheduleType>,

    #[serde(rename = "Value", default)]
    pub value: Option<JsonValue>,

    #[serde(rename = "Period", default)]
    pub period: Option<Numeric>,

    #[serde(rename = "Times", default)]
    pub times: Option<Vec<Numeric>>,

    #[serde(rename = "Values", default)]
    pub values: Option<Vec<JsonValue>>,

    #[serde(rename = "OffsetTime", default)]
    pub offset_time: Option<Numeric>,
}

impl ScheduleEntry {
    pub fn schedule_type(&self) -> ScheduleType {
        // ! Currently doesn't explicitly raise errors
        // if schedule contains fields it shouldn't
        // i.e. periodic shouldn't contain value field
        let schedule_type = if let Some(explicit) = &self.schedule_type {
            explicit.clone()
        } else {
            match (&self.value, &self.period, &self.times, &self.values) {
                (Some(_), None, None, None) => ScheduleType::Constant,
                (None, Some(_), Some(_), Some(_)) => ScheduleType::Periodic,
                (None, None, None, None) => ScheduleType::Default,
                _ => panic!("Error parsing schedule type"),
            }
        };

        match schedule_type {
            ScheduleType::Constant => {
                debug_assert!(self.value.is_some());
                debug_assert!(self.times.is_none());
                debug_assert!(self.period.is_none());
                debug_assert!(self.values.is_none());
            }
            ScheduleType::Default => {
                debug_assert!(self.value.is_none());
                debug_assert!(self.times.is_none());
                debug_assert!(self.period.is_none());
                debug_assert!(self.values.is_none());
            }
            ScheduleType::Periodic => {
                debug_assert!(self.value.is_none());
                debug_assert!(self.times.is_some());
                debug_assert!(self.period.is_some());
                debug_assert!(self.values.is_some());

                let period = f64::from(self.period.unwrap());
                if period == 24.0 {
                    debug_assert!(self.offset_time.is_none());
                }
            }
        }

        schedule_type
    }
}

// ------------------------- Metadata Section -------------------------

#[derive(Debug, Deserialize, Clone)]
pub struct ScheduleInfo {
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "Timezone", default)]
    pub timezone: i64,
    #[serde(rename = "StartDate")]
    pub start_date: String,
    #[serde(rename = "StartOffset")]
    pub start_offset: String,
    #[serde(rename = "ExperimentName")]
    pub experiment_name: String,
    #[serde(rename = "CabinetID")]
    pub cabinet_id: String,
    #[serde(rename = "User")]
    pub user: String,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "Parents")]
    pub parents: ScheduleParents,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ScheduleParents {
    #[serde(rename = "Primary")]
    pub primary: String,
    #[serde(rename = "Secondary")]
    pub secondary: Vec<String>,
}

// ------------------------- Top-level Container -------------------------

#[derive(Debug, Deserialize, Clone)]
pub struct ScheduleFile {
    #[serde(rename = "EventSchedules")]
    pub event_schedules: HashMap<String, JsonValue>, // Placeholder for now

    #[serde(rename = "VarTypeSpecs")]
    pub var_type_specs: HashMap<String, VariableTypeSpec>,

    #[serde(rename = "VariableSchedules")]
    pub variable_schedules: HashMap<String, ScheduleEntry>,

    #[serde(rename = "Info")]
    pub info: ScheduleInfo,
}
