use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

// Variable Type Specs

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

// Schedule Section

/// intermediate representation of variable schedule entries
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "ScheduleType", rename_all = "lowercase")]
pub enum ScheduleEntry {
    Constant {
        #[serde(rename = "VariableType")]
        variable_type: String,
        #[serde(rename = "Value")]
        value: JsonValue,
    },
    Periodic {
        #[serde(rename = "VariableType")]
        variable_type: String,
        #[serde(rename = "Period")]
        period: f64,
        #[serde(rename = "Times")]
        times: Vec<f64>,
        #[serde(rename = "Values")]
        values: Vec<JsonValue>,
        #[serde(rename = "OffsetTime", default)]
        offset_time: f64,
    },
    Default {
        #[serde(rename = "VariableType")]
        variable_type: String,
    },
}

impl ScheduleEntry {
    pub fn variable_type(&self) -> &str {
        match self {
            ScheduleEntry::Constant { variable_type, .. } => variable_type,
            ScheduleEntry::Periodic { variable_type, .. } => variable_type,
            ScheduleEntry::Default { variable_type } => variable_type,
        }
    }
}
// Extensions

#[derive(Debug, Deserialize, Clone)]
pub struct ExtensionNamespace {
    #[serde(rename = "VariableSchedules", default)]
    pub variable_schedules: HashMap<String, ScheduleEntry>,
    #[serde(flatten)]
    pub extra: HashMap<String, JsonValue>,
}

// Metadata Section

#[derive(Debug, Deserialize, Clone)]
pub struct ScheduleInfo {
    #[serde(rename = "Version")]
    pub _version: String,
    #[serde(rename = "Timezone", default)]
    pub timezone: i64,
    #[serde(rename = "StartDate")]
    pub start_date: String,
    #[serde(rename = "StartOffset")]
    pub start_offset: String,
    #[serde(rename = "ExperimentName")]
    pub experiment_name: String,
    #[serde(rename = "CabinetID")]
    pub _cabinet_id: String,
    #[serde(rename = "User")]
    pub _user: String,
    #[serde(rename = "Description")]
    pub _description: String,
    #[serde(rename = "Parents")]
    pub _parents: ScheduleParents,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ScheduleParents {
    #[serde(rename = "Primary")]
    pub _primary: String,
    #[serde(rename = "Secondary")]
    pub _secondary: Vec<String>,
}

// Top-level Container

#[derive(Debug, Deserialize, Clone)]
pub struct LunaLuz {
    #[serde(rename = "EventSchedules", default)]
    pub _event_schedules: Option<HashMap<String, JsonValue>>, // depreciated

    #[serde(rename = "VarTypeSpecs")]
    pub var_type_specs: HashMap<String, VariableTypeSpec>,

    #[serde(rename = "VariableSchedules")]
    pub variable_schedules: HashMap<String, ScheduleEntry>,

    #[serde(rename = "Info")]
    pub info: ScheduleInfo,

    #[serde(rename = "Extensions", default)] // should be empty hashmap if not included
    pub extensions: HashMap<String, ExtensionNamespace>,
}
