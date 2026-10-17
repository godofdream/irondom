use crate::domain::Device;
use crate::integrations::traits::Integration;
use async_trait::async_trait;
use serde_json::json;

pub struct Ziggurat {
    pub serial_port: String,
}

impl Ziggurat {
    pub fn from_config(cfg: &serde_json::Value) -> Self {
        Self {
            serial_port: cfg
                .get("serial_port")
                .and_then(|v| v.as_str())
                .unwrap_or("/dev/ttyUSB0")
                .into(),
        }
    }
}

#[async_trait]
impl Integration for Ziggurat {
    fn name(&self) -> &str {
        "ziggurat"
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
