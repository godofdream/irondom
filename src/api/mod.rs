use crate::app::AppState;
use crate::domain::{Automation, Device, Entity, Scene};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;

pub fn router(_state: AppState) -> Router<AppState> {
    Router::<AppState>::new()
        .route("/api/status", get(status))
        .route("/api/devices", get(list_devices).post(create_device))
        .route("/api/devices/:id", get(get_device).delete(delete_device))
        .route("/api/entities", get(list_entities).post(create_entity))
        .route("/api/automations", get(list_automations).post(create_automation))
        .route("/api/automations/:id/run", post(run_automation))
        .route("/api/scenes", get(list_scenes).post(create_scene))
        .route("/api/integrations", get(list_integrations))
}

async fn status(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({"status":"ok","version":"0.1.0","port": state.config.http_port}))
}

async fn list_devices(State(state): State<AppState>) -> Result<Json<Vec<Device>>, StatusCode> {
    state.db.list_devices().map(Json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
async fn get_device(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<Device>, StatusCode> {
    match state.db.get_device(&id) {
        Ok(Some(d)) => Ok(Json(d)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
async fn create_device(State(state): State<AppState>, Json(mut d): Json<Device>) -> Result<Json<Device>, StatusCode> {
    if d.id.is_empty() { d.id = uuid::Uuid::new_v4().to_string(); }
    if d.updated == 0 { d.updated = chrono::Utc::now().timestamp(); }
    state.db.upsert_device(&d).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(d))
}
async fn delete_device(State(state): State<AppState>, Path(id): Path<String>) -> StatusCode {
    if state.db.delete_device(&id).is_ok() { StatusCode::NO_CONTENT } else { StatusCode::INTERNAL_SERVER_ERROR }
}

async fn list_entities(State(state): State<AppState>) -> Result<Json<Vec<Entity>>, StatusCode> {
    state.db.list_entities().map(Json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
async fn create_entity(State(state): State<AppState>, Json(mut e): Json<Entity>) -> Result<Json<Entity>, StatusCode> {
    if e.id.is_empty() { e.id = uuid::Uuid::new_v4().to_string(); }
    if e.updated == 0 { e.updated = chrono::Utc::now().timestamp(); }
    state.db.upsert_entity(&e).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(e))
}

async fn list_automations(State(state): State<AppState>) -> Result<Json<Vec<Automation>>, StatusCode> {
    Ok(Json(state.automation_engine.list()))
}
async fn create_automation(State(state): State<AppState>, Json(mut a): Json<Automation>) -> Result<Json<Automation>, StatusCode> {
    if a.id.is_empty() { a.id = uuid::Uuid::new_v4().to_string(); }
    if a.updated == 0 { a.updated = chrono::Utc::now().timestamp(); }
    state.automation_engine.add(a.clone()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(a))
}
async fn run_automation(State(state): State<AppState>, Path(id): Path<String>) -> StatusCode {
    if state.automation_engine.run_one(&id).is_ok() { StatusCode::OK } else { StatusCode::INTERNAL_SERVER_ERROR }
}

async fn list_scenes(State(state): State<AppState>) -> Result<Json<Vec<Scene>>, StatusCode> {
    state.db.list_scenes().map(Json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
async fn create_scene(State(state): State<AppState>, Json(mut s): Json<Scene>) -> Result<Json<Scene>, StatusCode> {
    if s.id.is_empty() { s.id = uuid::Uuid::new_v4().to_string(); }
    if s.updated == 0 { s.updated = chrono::Utc::now().timestamp(); }
    state.db.upsert_scene(&s).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(s))
}

async fn list_integrations(State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({"integrations":["esphome","tesla_fleet","tesla_wall_connector","ziggurat","tuya"]}))
}
