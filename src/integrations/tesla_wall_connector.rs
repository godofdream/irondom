use crate::domain::Device;
use crate::integrations::traits::Integration;
use async_trait::async_trait;
use serde_json::json;

pub struct TeslaWallConnector {
    pub host: String,
}

impl TeslaWallConnector {
    pub fn from_config(cfg: &serde_json::Value) -> Self {
        Self {
            host: cfg
                .get("host")
                .and_then(|v| v.as_str())
                .unwrap_or("192.168.1.50")
                .into(),
        }
    }
}

#[async_trait]
impl Integration for TeslaWallConnector {
    fn name(&self) -> &str {
        "tesla_wall_connector"
    }
    async fn start(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
    async fn discover(&mut self) -> anyhow::Result<Vec<Device>> {
        Ok(vec![])
    }
    async fn execute(
        &self,
        _action: &str,
        _payload: &serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        Ok(json!({"ok": true}))
    }
}
