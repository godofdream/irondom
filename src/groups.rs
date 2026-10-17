use crate::db::Db;
use crate::domain::{DeviceLink, Group};
use crate::integrations;
use std::collections::{HashMap, HashSet};

pub fn create_group(db: &Db, name: String, members: Vec<String>) -> anyhow::Result<Group> {
    let mut seen = HashSet::new();
    let members: Vec<String> = members
        .into_iter()
        .filter(|m| seen.insert(m.clone()))
        .collect();

    let entities = db.list_entities()?;
    let devices = db.list_devices()?;
    let device_by_id: HashMap<String, _> = devices.into_iter().map(|d| (d.id.clone(), d)).collect();

    let mut integrations_of: Vec<Option<String>> = Vec::with_capacity(members.len());
    for m in &members {
        let integration = entities
            .iter()
            .find(|e| e.id == *m)
            .and_then(|e| e.device_id.clone())
            .and_then(|did| device_by_id.get(&did))
            .and_then(|d| d.integration.clone());
        integrations_of.push(integration);
    }

    let allowed = ["tuya", "ziggurat", "zigbee"];
    let common = if members.is_empty() {
        None
    } else {
        let first = &integrations_of[0];
        if first.is_none() {
            None
        } else if integrations_of.iter().all(|i| i == first) {
            first.clone()
        } else {
            None
        }
    };

    let (integration, native_id) = match common {
        Some(integration) if allowed.contains(&integration.as_str()) => {
            match integrations::grouping::try_native_group(&integration, &name, &members)? {
                Some(nid) => (Some(integration), Some(nid)),
                None => (None, None),
            }
        }
        _ => (None, None),
    };

    let group = Group {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        members,
        integration,
        native_id,
        created: chrono::Utc::now().timestamp(),
    };
    db.upsert_group(&group)?;
    Ok(group)
}

pub fn list_groups(db: &Db) -> anyhow::Result<Vec<Group>> {
    db.list_groups()
}

pub fn delete_group(db: &Db, id: &str) -> anyhow::Result<()> {
    db.delete_group(id)
}

pub fn link_devices(
    db: &Db,
    source: String,
    target: String,
    kind: String,
) -> anyhow::Result<DeviceLink> {
    let link = DeviceLink {
        id: uuid::Uuid::new_v4().to_string(),
        source: source.clone(),
        target: target.clone(),
        kind,
        created: chrono::Utc::now().timestamp(),
    };
    db.upsert_link(&link)?;

    let src_dev = db.get_device(&source)?;
    let tgt_dev = db.get_device(&target)?;
    if let (Some(s), Some(t)) = (src_dev, tgt_dev) {
        if s.integration.as_deref() == Some("ziggurat")
            && t.integration.as_deref() == Some("ziggurat")
        {
            let _ = integrations::grouping::bind_zigbee(&source, &target);
        }
    }

    Ok(link)
}

pub fn list_links(db: &Db) -> anyhow::Result<Vec<DeviceLink>> {
    db.list_links()
}

pub fn delete_link(db: &Db, id: &str) -> anyhow::Result<()> {
    db.delete_link(id)
}
