use serde::{Deserialize, Deserializer};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

// ------------------------- Variable Type Spec -------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum VarDataKind {
    Interval,
    Ratio,
    Nominal,
    Ordinal,
    Administrative,
}

#[derive(Debug, Deserialize)]
pub struct VariableTypeSpec {
    #[serde(rename = "VariableType")]
    pub kind: VarDataKind,

    #[serde(rename = "DefaultValue")]
    pub default: JsonValue,

    #[serde(rename = "Description")]
    pub description: String,

    #[serde(rename = "Categories", default)]
    pub categories: Option<Vec<String>>,
}

// ------------------------- Schedule Section -------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScheduleType {
    Constant,
    Periodic,
    Default,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct ScheduleEntry {
    #[serde(rename = "VariableType")]
    pub variable_type: String,

    #[serde(rename = "ScheduleType", default)]
    pub schedule_type: Option<ScheduleType>,

    #[serde(default)]
    pub value: Option<JsonValue>,

    #[serde(default)]
    pub period: Option<Numeric>,

    #[serde(default)]
    pub times: Option<Vec<Numeric>>,

    #[serde(default)]
    pub values: Option<Vec<JsonValue>>,
}

// ------------------------- Metadata Section -------------------------

#[derive(Debug, Deserialize)]
pub struct ScheduleInfo {
    #[serde(rename = "Version")]
    pub version: String,
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

#[derive(Debug, Deserialize)]
pub struct ScheduleParents {
    #[serde(rename = "Primary")]
    pub primary: String,
    #[serde(rename = "Secondary")]
    pub secondary: Vec<String>,
}

// ------------------------- Top-level Container -------------------------

#[derive(Debug, Deserialize)]
pub struct ScheduleFile {
    #[serde(rename = "EventSchedules")]
    pub event_schedules: HashMap<String, JsonValue>, // Placeholder for now

    #[serde(rename = "VariableTypeSpecs")]
    pub variable_type_specs: HashMap<String, VariableTypeSpec>,

    #[serde(rename = "VariableSchedules")]
    pub variable_schedules: HashMap<String, ScheduleEntry>,

    #[serde(rename = "Info")]
    pub info: ScheduleInfo,
}
