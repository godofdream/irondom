use crate::domain::Device;
use crate::integrations::traits::Integration;
use async_trait::async_trait;
use serde_json::json;

pub struct TeslaFleet {
    pub client_id: String,
    pub client_secret: String,
    pub region: String,
}

impl TeslaFleet {
    pub fn from_config(cfg: &serde_json::Value) -> Self {
        Self {
            client_id: cfg.get("client_id").and_then(|v| v.as_str()).unwrap_or("").into(),
            client_secret: cfg.get("client_secret").and_then(|v| v.as_str()).unwrap_or("").into(),
            region: cfg.get("region").and_then(|v| v.as_str()).unwrap_or("na").into(),
        }
    }
}

#[async_trait]
impl Integration for TeslaFleet {
    fn name(&self) -> &str { "tesla_fleet" }
    async fn start(&mut self) -> anyhow::Result<()> { Ok(()) }
    async fn discover(&mut self) -> anyhow::Result<Vec<Device>> { Ok(vec![]) }
    async fn execute(&self, _action: &str, _payload: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
        Ok(json!({"ok": true}))
    }
}
