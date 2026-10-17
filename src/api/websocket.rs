use crate::app::AppState;
use axum::extract::ws::{Message, WebSocket};

use serde_json::{json, Value};

pub async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // auth required
    let _ = socket.send(Message::Text(json!({"type":"auth_required","ha_version":"2024.1.0"}).to_string())).await;
    let mut authenticated = false;
    let mut rx = state.event_tx.subscribe();
    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(t))) => {
                        if let Ok(v) = serde_json::from_str::<Value>(&t) {
                            let msg_type = v.get("type").and_then(|x| x.as_str()).unwrap_or("");
                            if !authenticated {
                                if msg_type == "auth" {
                                    let token = v.get("access_token").and_then(|x| x.as_str()).unwrap_or("");
                                    let ok = state.config.api_token.as_ref().map(|t| t == token).unwrap_or(true);
                                    if ok {
                                        authenticated = true;
                                        let _ = socket.send(Message::Text(json!({"type":"auth_ok","ha_version":"2024.1.0"}).to_string())).await;
                                    } else {
                                        let _ = socket.send(Message::Text(json!({"type":"auth_invalid","message":"invalid token"}).to_string())).await;
                                    }
                                }
                                continue;
                            }
                            let id = v.get("id").cloned().unwrap_or(json!(null));
                            match msg_type {
                                "ping" => {
                                    let _ = socket.send(Message::Text(json!({"type":"pong","id":id}).to_string())).await;
                                }
                                "get_states" => {
                                    let states: Vec<Value> = state.db.list_entities().unwrap_or_default().iter().map(|e| {
                                        let attrs: Value = serde_json::from_str(&e.attributes).unwrap_or(json!({}));
                                        json!({"entity_id":e.id,"state":e.value.clone().unwrap_or("unknown".into()),"attributes":attrs,"last_changed":e.updated,"last_updated":e.updated,"context":{"id":e.id,"parent_id":null,"user_id":null}})
                                    }).collect();
                                    let _ = socket.send(Message::Text(json!({"type":"result","id":id,"success":true,"result":states}).to_string())).await;
                                }
                                "get_services" => {
                                    let _ = socket.send(Message::Text(json!({"type":"result","id":id,"success":true,"result":{}}).to_string())).await;
                                }
                                "get_config" => {
                                    let _ = socket.send(Message::Text(json!({"type":"result","id":id,"success":true,"result":{"location_name":"Irondom","version":"2024.1.0"}}).to_string())).await;
                                }
                                "call_service" => {
                                    let domain = v.get("domain").and_then(|x| x.as_str()).unwrap_or("");
                                    let service = v.get("service").and_then(|x| x.as_str()).unwrap_or("");
                                    let _ = crate::api::homeassistant::call_service_internal(&state, domain, service, v.get("service_data").cloned().unwrap_or(json!({}))).await;
                                    let _ = socket.send(Message::Text(json!({"type":"result","id":id,"success":true,"result":null}).to_string())).await;
                                }
                                "subscribe_events" => {
                                    let _ = socket.send(Message::Text(json!({"type":"result","id":id,"success":true,"result":null}).to_string())).await;
                                }
                                _ => {
                                    let _ = socket.send(Message::Text(json!({"type":"result","id":id,"success":false,"error":{"code":"unknown_command","message":msg_type}}).to_string())).await;
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            evt = rx.recv() => {
                if let Ok(e) = evt {
                    let _ = socket.send(Message::Text(e.to_string())).await;
                }
            }
        }
    }
}
