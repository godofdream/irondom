use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLink {
    pub id: String,
    pub source: String,
    pub target: String,
    pub kind: String,
    pub created: i64,
}