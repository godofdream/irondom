use crate::domain::Automation;
use serde_yaml::Value;
use uuid::Uuid;

pub fn parse_ha_yaml(input: &str) -> anyhow::Result<Vec<Automation>> {
    let v: Value = serde_yaml::from_str(input)?;
    let seq = v.as_sequence().cloned().unwrap_or_default();
    let mut out = Vec::new();
    for item in seq {
        let name = item.get("alias").and_then(|x| x.as_str()).unwrap_or("automation").to_string();
        let trigger = item.get("trigger").cloned().unwrap_or(Value::Null);
        let action = item.get("action").cloned().unwrap_or(Value::Null);
        let trigger_yaml = serde_yaml::to_string(&trigger)?;
        let action_yaml = serde_yaml::to_string(&action)?;
        out.push(Automation {
            id: Uuid::new_v4().to_string(),
            name,
            trigger_yaml,
            action_yaml,
            enabled: item.get("initial_state").and_then(|x| x.as_bool()).unwrap_or(true),
            updated: chrono::Utc::now().timestamp(),
        });
    }
    Ok(out)
}
