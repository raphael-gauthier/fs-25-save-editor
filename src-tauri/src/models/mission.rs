use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MissionStatus {
    Created,
    Running,
    Completed,
}

impl MissionStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "1" => MissionStatus::Running,
            "2" => MissionStatus::Completed,
            _ => MissionStatus::Created,
        }
    }

    pub fn to_xml_str(&self) -> &str {
        match self {
            MissionStatus::Created => "0",
            MissionStatus::Running => "1",
            MissionStatus::Completed => "2",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mission {
    pub unique_id: String,
    pub mission_type: String,
    pub status: MissionStatus,
    pub reward: f64,
    pub reimbursement: f64,
    pub completion: f64,
    pub field_id: Option<u32>,
    pub fruit_type: Option<String>,
    pub expected_liters: Option<f64>,
    pub deposited_liters: Option<f64>,
}
