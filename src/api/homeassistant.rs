use crate::app::AppState;
use crate::domain::Entity;
use axum::extract::ws::WebSocketUpgrade;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

fn now_ts() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

fn ts_to_iso(ts: i64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0)
        .map(|d| d.to_rfc3339())
        .unwrap_or_default()
}

fn entity_to_state(e: &Entity) -> Value {
    let attrs: Value = serde_json::from_str(&e.attributes).unwrap_or(json!({}));
    json!({
        "entity_id": e.id,
        "state": e.value.clone().unwrap_or_else(|| "unknown".to_string()),
        "attributes": attrs,
        "last_changed": ts_to_iso(e.updated),
        "last_updated": ts_to_iso(e.updated),
        "context": {"id": e.id, "parent_id": null, "user_id": null}
    })
}

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/api/", get(api_root))
        .route("/api/config", get(api_config))
        .route("/api/discovery_info", get(discovery_info))
        .route("/api/components", get(components))
        .route("/api/events", get(list_events))
        .route("/api/events/:event_type", post(fire_event))
        .route("/api/services", get(list_services))
        .route("/api/services/:domain/:service", post(call_service))
        .route("/api/states", get(list_states))
        .route("/api/states/:entity_id", get(get_state).post(set_state).delete(delete_state))
        .route("/api/history/period/:timestamp", get(history_period))
        .route("/api/logbook/:timestamp", get(logbook))
        .route("/api/error_log", get(error_log))
        .route("/api/template", post(render_template))
        .route("/api/onboarding", get(onboarding))
        .route("/api/websocket", get(ws_upgrade))
}

async fn api_root() -> Json<Value> {
    Json(json!({"message":"API running."}))
}

async fn api_config(State(state): State<AppState>) -> Json<Value> {
    let components: Vec<String> = state.config.integrations_list();
    Json(json!({
        "location_name": "Irondom",
        "latitude": 0.0,
        "longitude": 0.0,
        "elevation": 0,
        "unit_system": {"length":"km","mass":"g","temperature":"C","volume":"L"},
        "time_zone": "UTC",
        "components": components,
        "version": "2024.1.0",
        "theme": "default",
        "state": "RUNNING",
        "go2rtc_url": state.config.go2rtc_url.clone().unwrap_or_default()
    }))
}

async fn discovery_info(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "base_url": format!("http://0.0.0.0:{}", state.config.http_port),
        "location_name": "Irondom",
        "requires_api_password": false,
        "version": "2024.1.0"
    }))
}

async fn components(State(state): State<AppState>) -> Json<Value> {
    Json(json!(state.config.integrations_list()))
}

async fn list_events() -> Json<Value> {
    Json(json!(["state_changed","service_executed","automation_triggered"]))
}

async fn fire_event(State(state): State<AppState>, Path(event_type): Path<String>, Json(body): Json<Value>) -> StatusCode {
    let evt = json!({"event_type": event_type, "data": body, "time_fired": ts_to_iso(now_ts())});
    let _ = state.event_tx.send(json!({"type":"event","event": evt}));
    StatusCode::OK
}

async fn list_services() -> Json<Value> {
    Json(json!({
        "light": [{"domain":"light","service":"turn_on","name":"Turn on","description":"","fields":{}},
                   {"domain":"light","service":"turn_off","name":"Turn off","description":"","fields":{}}],
        "switch": [{"domain":"switch","service":"turn_on","name":"Turn on","description":"","fields":{}},
                   {"domain":"switch","service":"turn_off","name":"Turn off","description":"","fields":{}}],
        "climate": [{"domain":"climate","service":"set_temperature","name":"Set temperature","description":"","fields":{}}],
        "media_player": [{"domain":"media_player","service":"volume_set","name":"Volume set","description":"","fields":{}}],
        "group": [{"domain":"group","service":"turn_on","name":"Turn on","description":"","fields":{}},
                  {"domain":"group","service":"turn_off","name":"Turn off","description":"","fields":{}}]
    }))
}

pub async fn call_service_internal(state: &AppState, domain: &str, service: &str, body: Value) -> Vec<Value> {
    let mut changed: Vec<Value> = Vec::new();
    let entity_id = body.get("entity_id").and_then(|v| v.as_str()).map(|s| s.to_string());
    if domain == "group" {
        if let Some(id) = &entity_id {
            if let Ok(groups) = state.db.list_groups() {
                if let Some(g) = groups.into_iter().find(|g| g.id == *id) {
                    for m in g.members {
                        apply_service(state, &domain_for_entity(&m), service, &m, &body).await;
                        if let Ok(Some(e)) = state.db.get_entity(&m) { changed.push(entity_to_state(&e)); }
                    }
                }
            }
        }
        let _ = state.event_tx.send(json!({"type":"event","event":{"event_type":"service_executed","data":{"domain":domain,"service":service}}}));
        return changed;
    }
    if let Some(id) = entity_id {
        let ent_domain = domain_for_entity(&id);
        apply_service(state, &ent_domain, service, &id, &body).await;
        if let Ok(Some(e)) = state.db.get_entity(&id) {
            state.db.record_history(&e.id, e.value.clone().unwrap_or_default(), e.attributes.clone()).ok();
            let _ = state.event_tx.send(json!({"type":"event","event":{"event_type":"state_changed","data":{"entity_id":e.id,"new_state":e.value}}}));
            changed.push(entity_to_state(&e));
        }
    }
    let _ = state.event_tx.send(json!({"type":"event","event":{"event_type":"service_executed","data":{"domain":domain,"service":service}}}));
    changed
}

async fn call_service(State(state): State<AppState>, Path((domain, service)): Path<(String,String)>, Json(body): Json<Value>) -> Result<Json<Vec<Value>>, StatusCode> {
    let changed = call_service_internal(&state, &domain, &service, body).await;
    Ok(Json(changed))
}

fn domain_for_entity(entity_id: &str) -> String {
    entity_id.split('.').next().unwrap_or("").to_string()
}

async fn apply_service(state: &AppState, _domain: &str, service: &str, entity_id: &str, body: &Value) {
    if let Ok(Some(mut e)) = state.db.get_entity(entity_id) {
        match service {
            "turn_on" => { e.value = Some("on".into()); }
            "turn_off" => { e.value = Some("off".into()); }
            "toggle" => { e.value = Some(if e.value.as_deref() == Some("on") { "off" } else { "on" }.into()); }
            "set_temperature" => {
                if let Some(t) = body.get("temperature") { e.value = Some(t.to_string()); }
            }
            "set_hvac_mode" => {
                if let Some(m) = body.get("hvac_mode").and_then(|v| v.as_str()) { e.value = Some(m.into()); }
            }
            "volume_set" => {
                if let Some(v) = body.get("volume_level") { 
                    let mut attrs: Value = serde_json::from_str(&e.attributes).unwrap_or(json!({}));
                    attrs["volume_level"] = v.clone();
                    e.attributes = attrs.to_string();
                }
            }
            _ => {}
        }
        e.updated = now_ts();
        let _ = state.db.upsert_entity(&e);
    }
}

async fn list_states(State(state): State<AppState>) -> Result<Json<Vec<Value>>, StatusCode> {
    match state.db.list_entities() {
        Ok(ents) => Ok(Json(ents.iter().map(entity_to_state).collect())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_state(State(state): State<AppState>, Path(entity_id): Path<String>) -> Result<Json<Value>, StatusCode> {
    match state.db.get_entity(&entity_id) {
        Ok(Some(e)) => Ok(Json(entity_to_state(&e))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn set_state(State(state): State<AppState>, Path(entity_id): Path<String>, Json(body): Json<Value>) -> Result<Json<Value>, StatusCode> {
    let mut e = state.db.get_entity(&entity_id).ok().flatten().unwrap_or(Entity {
        id: entity_id.clone(),
        device_id: None,
        name: body.get("attributes").and_then(|a| a.get("friendly_name")).and_then(|v| v.as_str()).unwrap_or(&entity_id).to_string(),
        kind: domain_for_entity(&entity_id),
        value: None,
        attributes: "{}".into(),
        updated: 0,
    });
    if let Some(s) = body.get("state").and_then(|v| v.as_str()) { e.value = Some(s.to_string()); }
    if let Some(a) = body.get("attributes") { e.attributes = a.to_string(); }
    e.updated = now_ts();
    state.db.upsert_entity(&e).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.db.record_history(&e.id, e.value.clone().unwrap_or_default(), e.attributes.clone()).ok();
    let _ = state.event_tx.send(json!({"type":"event","event":{"event_type":"state_changed","data":{"entity_id":e.id,"new_state":e.value}}}));
    Ok(Json(entity_to_state(&e)))
}

async fn delete_state(State(state): State<AppState>, Path(entity_id): Path<String>) -> StatusCode {
    if state.db.delete_entity(&entity_id).is_ok() { StatusCode::OK } else { StatusCode::INTERNAL_SERVER_ERROR }
}

async fn history_period(State(state): State<AppState>, Path(timestamp): Path<String>) -> Result<Json<Vec<Vec<Value>>>, StatusCode> {
    // parse timestamp loosely: ignore and use 0
    let _ = timestamp;
    let rows = state.db.list_history(None, 0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // group by entity
    let mut map: std::collections::BTreeMap<String, Vec<Value>> = std::collections::BTreeMap::new();
    for (eid, st, attrs, ts) in rows {
        let e = Entity { id: eid.clone(), device_id: None, name: eid.clone(), kind: domain_for_entity(&eid), value: Some(st.clone()), attributes: attrs.clone(), updated: ts };
        map.entry(eid).or_default().push(entity_to_state(&e));
    }
    Ok(Json(map.into_values().collect()))
}

async fn logbook(State(_state): State<AppState>, Path(_timestamp): Path<String>) -> Json<Value> {
    Json(json!([]))
}

async fn error_log() -> String {
    String::new()
}

async fn render_template(Json(body): Json<Value>) -> Json<Value> {
    let tpl = body.get("template").and_then(|v| v.as_str()).unwrap_or("");
    // minimal: replace {{ states('entity') }} 
    let mut out = tpl.to_string();
    // naive replacement
    if let Some(start) = out.find("states('") {
        if let Some(end) = out[start..].find("')") {
            let ent = &out[start+8..start+end];
            // can't access db here; return placeholder
            out = out.replace(&format!("{{{{ states('{}') }}}}", ent), "unknown");
        }
    }
    Json(json!({"result": out}))
}

async fn onboarding() -> Json<Value> {
    Json(json!({"onboarding_done": true}))
}

async fn ws_upgrade(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| crate::api::websocket::handle_socket(socket, state))
}

// Config helper
trait ConfigExt { fn integrations_list(&self) -> Vec<String>; }
impl ConfigExt for crate::config::Config {
    fn integrations_list(&self) -> Vec<String> {
        vec!["light".into(),"switch".into(),"sensor".into(),"climate".into(),"media_player".into(),"camera".into(),"group".into(),"automation".into(),"esphome".into(),"tesla_fleet".into(),"tesla_wall_connector".into(),"ziggurat".into(),"tuya".into(),"lg_webos".into()]
    }
}
