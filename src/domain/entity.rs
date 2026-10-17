use serde::{Deserialize, Serialize};

fn default_attributes() -> String {
    "{}".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub device_id: Option<String>,
    pub name: String,
    pub kind: String,
    pub value: Option<String>,
    #[serde(default = "default_attributes")]
    pub attributes: String,
    pub updated: i64,
}