use crate::domain::Device;
use async_trait::async_trait;
#[async_trait]
pub trait Integration: Send + Sync {
    fn name(&self) -> &str;
    async fn start(&mut self) -> anyhow::Result<()>;
    async fn discover(&mut self) -> anyhow::Result<Vec<Device>>;
    async fn execute(&self, action: &str, payload: &serde_json::Value) -> anyhow::Result<serde_json::Value>;
}

pub struct IntegrationManager {
    pub items: Vec<Box<dyn Integration + Send + Sync>>,
}

impl IntegrationManager {
    pub fn new() -> Self { Self { items: Vec::new() } }
    pub fn register(&mut self, i: Box<dyn Integration + Send + Sync>) { self.items.push(i); }
    pub fn names(&self) -> Vec<String> { self.items.iter().map(|i| i.name().to_string()).collect() }
    pub async fn start_all(&mut self) -> anyhow::Result<()> {
        for i in self.items.iter_mut() { i.start().await?; }
        Ok(())
    }
    pub async fn discover_all(&mut self) -> anyhow::Result<Vec<Device>> {
        let mut out = Vec::new();
        for i in self.items.iter_mut() { out.extend(i.discover().await?); }
        Ok(out)
    }
}

impl Default for IntegrationManager {
    fn default() -> Self { Self::new() }
}
