use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub integration: Option<String>,
    pub state: Option<String>,
    pub updated: i64,
}

impl Device {
    pub fn new(id: String, name: String, kind: String, integration: Option<String>) -> Self {
        Self {
            id,
            name,
            kind,
            integration,
            state: None,
            updated: chrono::Utc::now().timestamp(),
        }
    }
}
