use crate::automation::engine::AutomationEngine;
use std::time::Duration;

pub async fn run_scheduler(engine: AutomationEngine) {
    loop {
        if let Err(e) = engine.evaluate() {
            tracing::warn!("automation evaluate error: {}", e);
        }
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}

pub fn spawn(engine: AutomationEngine) {
    tokio::spawn(run_scheduler(engine));
}
