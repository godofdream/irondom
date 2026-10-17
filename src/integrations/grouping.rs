use uuid::Uuid;

pub fn try_native_group(
    integration: &str,
    name: &str,
    members: &[String],
) -> anyhow::Result<Option<String>> {
    let _ = (name, members);
    match integration {
        "tuya" => Ok(Some(format!("tuya_group_{}", Uuid::new_v4()))),
        "ziggurat" | "zigbee" => Ok(Some(format!("zigbee_group_{}", Uuid::new_v4()))),
        _ => Ok(None),
    }
}

pub fn bind_zigbee(source: &str, target: &str) -> anyhow::Result<()> {
    let _ = (source, target);
    Ok(())
}
