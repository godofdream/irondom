use crate::domain::Device;
use crate::integrations::traits::Integration;
use async_trait::async_trait;
use serde_json::json;

pub struct EspHome {
    pub host: String,
    pub port: u16,
}

impl EspHome {
    pub fn from_config(cfg: &serde_json::Value) -> Self {
        Self {
            host: cfg.get("host").and_then(|v| v.as_str()).unwrap_or("127.0.0.1").into(),
            port: cfg.get("port").and_then(|v| v.as_u64()).unwrap_or(6053) as u16,
        }
    }
}

#[async_trait]
impl Integration for EspHome {
    fn name(&self) -> &str { "esphome" }
    async fn start(&mut self) -> anyhow::Result<()> { Ok(()) }
    async fn discover(&mut self) -> anyhow::Result<Vec<Device>> { Ok(vec![]) }
    async fn execute(&self, _action: &str, _payload: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
        Ok(json!({"ok": true}))
    }
}
