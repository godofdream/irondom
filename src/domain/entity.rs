use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub device_id: Option<String>,
    pub name: String,
    pub kind: String,
    pub value: Option<String>,
    pub updated: i64,
}
