use crate::automation::AutomationEngine;
use crate::config::Config;
use crate::db::Db;
use axum::{routing::get, Router};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: Db,
    pub automation_engine: AutomationEngine,
}

pub async fn build_state(config: Config) -> anyhow::Result<AppState> {
    let db = Db::open(&config.data_dir)?;
    let engine = AutomationEngine::new(db.clone());
    Ok(AppState { config, db, automation_engine: engine })
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(crate::api::router(state.clone()))
        .merge(crate::web::router())
        .with_state(state)
}
