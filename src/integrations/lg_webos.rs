use crate::domain::Device;
use crate::integrations::traits::Integration;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

pub struct LgWebOs {
    pub host: String,
    pub port: u16,
    pub client_key: Arc<Mutex<Option<String>>>,
}

impl LgWebOs {
    pub fn from_config(cfg: &Value) -> Self {
        Self {
            host: cfg.get("host").and_then(|v| v.as_str()).unwrap_or("192.168.0.10").into(),
            port: cfg.get("port").and_then(|v| v.as_u64()).unwrap_or(3000) as u16,
            client_key: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn connect(&self) -> anyhow::Result<()> {
        use futures::SinkExt;
        use futures::StreamExt;
        use tokio_tungstenite::connect_async;
        let url = format!("ws://{}:{}", self.host, self.port);
        let (mut ws, _) = connect_async(&url).await?;
        let reg = json!({
            "type": "register",
            "id": "register_0",
            "payload": {
                "forcePairing": false,
                "pairingType": "PROMPT",
                "manifest": {
                    "manifestVersion": 1,
                    "appVersion": "1.0",
                    "signed": {"created":"20240101","appId":"com.irondom.remote","vendorId":"irondom"},
                    "localizedAppNames": {"": "Irondom"},
                    "permissions": ["LAUNCH","LAUNCH_WEBAPP","APP_TO_APP","CLOSE","TEST_OPEN","TEST_PROTECTED","CONTROL_AUDIO","CONTROL_DISPLAY","CONTROL_INPUT_JOYSTICK","CONTROL_INPUT_MEDIA_RECORDING","CONTROL_INPUT_MEDIA_PLAYBACK","CONTROL_INPUT_TV","CONTROL_POWER","READ_APP_STATUS","READ_CURRENT_CHANNEL","READ_INPUT_DEVICE_LIST","READ_NETWORK_STATE","READ_RUNNING_APPS","READ_TV_CHANNEL_LIST","WRITE_NOTIFICATION_TOAST","READ_POWER_STATE","READ_COUNTRY_INFO"]
                }
            }
        });
        ws.send(Message::Text(reg.to_string())).await?;
        // read a few messages to capture client-key
        for _ in 0..5 {
            if let Some(Ok(Message::Text(t))) = ws.next().await {
                if let Ok(v) = serde_json::from_str::<Value>(&t) {
                    if let Some(k) = v.get("payload").and_then(|p| p.get("client-key")).and_then(|x| x.as_str()) {
                        *self.client_key.lock().await = Some(k.to_string());
                    }
                }
            } else { break; }
        }
        let _ = ws.close(None).await;
        Ok(())
    }

    pub async fn send_key(&self, key: &str) -> anyhow::Result<()> {
        self.pointer_action(json!({"type":"button","name": key})).await
    }

    pub async fn move_pointer(&self, dx: i32, dy: i32) -> anyhow::Result<()> {
        self.pointer_action(json!({"type":"move","dx": dx, "dy": dy})).await
    }

    pub async fn click(&self) -> anyhow::Result<()> {
        self.pointer_action(json!({"type":"click"})).await
    }

    async fn pointer_action(&self, msg: Value) -> anyhow::Result<()> {
        use futures::SinkExt;
        use futures::StreamExt;
        use tokio_tungstenite::connect_async;
        // get pointer socket path
        let url = format!("ws://{}:{}", self.host, self.port);
        let (mut ws, _) = connect_async(&url).await?;
        let req = json!({
            "type": "request",
            "id": "ptr_0",
            "uri": "ssap://com.webos.service.networkinput/getPointerInputSocket"
        });
        ws.send(Message::Text(req.to_string())).await?;
        let mut socket_path: Option<String> = None;
        for _ in 0..5 {
            if let Some(Ok(Message::Text(t))) = ws.next().await {
                if let Ok(v) = serde_json::from_str::<Value>(&t) {
                    if let Some(p) = v.get("payload").and_then(|x| x.get("socketPath")).and_then(|x| x.as_str()) {
                        socket_path = Some(p.to_string());
                        break;
                    }
                }
            } else { break; }
        }
        let _ = ws.close(None).await;
        let path = socket_path.ok_or_else(|| anyhow::anyhow!("no pointer socket"))?;
        let wsurl = format!("ws://{}:{}{}", self.host, self.port, path);
        let (mut pws, _) = connect_async(&wsurl).await?;
        pws.send(Message::Text(msg.to_string())).await?;
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let _ = pws.close(None).await;
        Ok(())
    }
}

#[async_trait]
impl Integration for LgWebOs {
    fn name(&self) -> &str { "lg_webos" }
    async fn start(&mut self) -> anyhow::Result<()> { Ok(()) }
    async fn discover(&mut self) -> anyhow::Result<Vec<Device>> {
        Ok(vec![Device::new(
            format!("media_player.lg_{}", self.host.replace('.',"_" )),
            format!("LG TV {}", self.host),
            "tv".into(),
            Some("lg_webos".into()),
        )])
    }
    async fn execute(&self, action: &str, payload: &Value) -> anyhow::Result<Value> {
        match action {
            "connect" => { self.connect().await?; Ok(json!({"ok": true})) }
            "key" => {
                let key = payload.get("key").and_then(|v| v.as_str()).unwrap_or("ENTER");
                self.send_key(key).await?;
                Ok(json!({"ok": true}))
            }
            "move" => {
                let dx = payload.get("dx").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let dy = payload.get("dy").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                self.move_pointer(dx, dy).await?;
                Ok(json!({"ok": true}))
            }
            "click" => { self.click().await?; Ok(json!({"ok": true})) }
            _ => Ok(json!({"ok": false, "error": "unknown action"})),
        }
    }
}
