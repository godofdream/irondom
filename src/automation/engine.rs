use crate::db::Db;
use crate::domain::Automation;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AutomationEngine {
    db: Db,
    cache: Arc<Mutex<Vec<Automation>>>,
}

impl AutomationEngine {
    pub fn new(db: Db) -> Self {
        let cache = db.list_automations().unwrap_or_default();
        Self { db, cache: Arc::new(Mutex::new(cache)) }
    }

    pub fn list(&self) -> Vec<Automation> {
        self.cache.lock().unwrap().clone()
    }

    pub fn add(&self, a: Automation) -> anyhow::Result<()> {
        self.db.upsert_automation(&a)?;
        let mut c = self.cache.lock().unwrap();
        if let Some(pos) = c.iter().position(|x| x.id == a.id) {
            c[pos] = a;
        } else {
            c.push(a);
        }
        Ok(())
    }

    pub fn remove(&self, id: &str) -> anyhow::Result<()> {
        self.db.delete_automation(id)?;
        let mut c = self.cache.lock().unwrap();
        c.retain(|x| x.id != id);
        Ok(())
    }

    pub fn run_one(&self, id: &str) -> anyhow::Result<()> {
        let c = self.cache.lock().unwrap();
        if let Some(a) = c.iter().find(|x| x.id == id) {
            tracing::info!("running automation {} action: {}", a.name, a.action_yaml);
        }
        Ok(())
    }

    pub fn evaluate(&self) -> anyhow::Result<()> {
        let now = chrono::Utc::now().format("%H:%M").to_string();
        let list = self.list();
        for a in list {
            if !a.enabled { continue; }
            if a.trigger_yaml.contains(&now) {
                let _ = self.run_one(&a.id);
            }
        }
        Ok(())
    }
}
