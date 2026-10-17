use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Automation {
    pub id: String,
    pub name: String,
    pub trigger_yaml: String,
    pub action_yaml: String,
    pub enabled: bool,
    pub updated: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "platform", rename_all = "snake_case")]
pub enum Trigger {
    Time { at: String },
    DeviceState { device_id: String, value: String },
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    SetState {
        entity_id: String,
        value: String,
    },
    CallService {
        service: String,
        data: serde_json::Value,
    },
    Delay {
        seconds: u64,
    },
}
