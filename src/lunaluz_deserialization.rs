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

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct ScheduleHeader {
    pub variable_type: String,
    #[serde(default)]
    pub schedule_type: Option<ScheduleType>,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScheduleType {
    Constant,
    Periodic,
    Default,
}

/// intermediate representation of variable schedule entries
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ScheduleEntry {
    Constant {
        #[serde(flatten)]
        header: ScheduleHeader,
        #[serde(rename = "Value")]
        value: JsonValue,
    },
    Periodic {
        #[serde(flatten)]
        header: ScheduleHeader,
        #[serde(rename = "Period")]
        period: f64,
        #[serde(rename = "Times")]
        times: Vec<f64>,
        #[serde(rename = "Values")]
        values: Vec<JsonValue>,
        #[serde(rename = "OffsetTime", default)]
        offset_time: Option<f64>,
    },
    Default {
        #[serde(flatten)]
        header: ScheduleHeader,
    },
}

impl ScheduleEntry {
    fn header(&self) -> &ScheduleHeader {
        match self {
            ScheduleEntry::Constant { header, .. } => &header,
            ScheduleEntry::Periodic { header, .. } => &header,
            ScheduleEntry::Default { header } => &header,
        }
    }

    pub fn schedule_type_unchecked(&self) -> &ScheduleType {
        match self {
            ScheduleEntry::Constant { .. } => &ScheduleType::Constant,
            ScheduleEntry::Periodic { .. } => &ScheduleType::Periodic,
            ScheduleEntry::Default { .. } => &ScheduleType::Default,
        }
    }

    pub fn variable_type(&self) -> &str {
        &self.header().variable_type
    }

    pub fn schedule_type(&self) -> Result<ScheduleType, String> {
        let inferred_type = self.schedule_type_unchecked();
        if let Some(specified) = self.header().schedule_type {
            if inferred_type != &specified {
                let var_type = self.variable_type();
                return Err(format!(
                    "Fields of '{}' do not match specified schedule type ({:?}); {:?} schedule was inferred",
                    var_type,
                    specified,
                    inferred_type
                ));
            }
        }

        Ok(inferred_type.to_owned())
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
