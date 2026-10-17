use crate::domain::Device;
use crate::integrations::traits::Integration;
use async_trait::async_trait;
use serde_json::json;

pub struct Tuya {
    pub access_id: String,
    pub access_secret: String,
    pub region: String,
}

impl Tuya {
    pub fn from_config(cfg: &serde_json::Value) -> Self {
        Self {
            access_id: cfg.get("access_id").and_then(|v| v.as_str()).unwrap_or("").into(),
            access_secret: cfg.get("access_secret").and_then(|v| v.as_str()).unwrap_or("").into(),
            region: cfg.get("region").and_then(|v| v.as_str()).unwrap_or("eu").into(),
        }
    }
}

#[async_trait]
impl Integration for Tuya {
    fn name(&self) -> &str { "tuya" }
    async fn start(&mut self) -> anyhow::Result<()> { Ok(()) }
    async fn discover(&mut self) -> anyhow::Result<Vec<Device>> { Ok(vec![]) }
    async fn execute(&self, _action: &str, _payload: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
        Ok(json!({"ok": true}))
    }
}
