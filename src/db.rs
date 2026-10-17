use crate::domain::{Automation, Device, DeviceLink, Entity, Group, Scene};
use rusqlite::{params, Connection, OptionalExtension};
use std::sync::{Arc, Mutex};

pub trait ToNullableString {
    fn to_nullable(self) -> Option<String>;
}

impl ToNullableString for &str {
    fn to_nullable(self) -> Option<String> {
        Some(self.to_string())
    }
}
impl ToNullableString for String {
    fn to_nullable(self) -> Option<String> {
        Some(self)
    }
}
impl ToNullableString for &String {
    fn to_nullable(self) -> Option<String> {
        Some(self.clone())
    }
}
impl ToNullableString for Option<&str> {
    fn to_nullable(self) -> Option<String> {
        self.map(|s| s.to_string())
    }
}

#[derive(Clone)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

impl Db {
    pub fn open(data_dir: &str) -> anyhow::Result<Self> {
        std::fs::create_dir_all(data_dir)?;
        let path = std::path::Path::new(data_dir).join("irondom.db");
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS devices (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                integration TEXT,
                state TEXT,
                updated INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS entities (
                id TEXT PRIMARY KEY,
                device_id TEXT,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                value TEXT,
                updated INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                members TEXT NOT NULL DEFAULT '[]',
                integration TEXT,
                native_id TEXT,
                created INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS device_links (
                id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                target TEXT NOT NULL,
                kind TEXT NOT NULL,
                created INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS automations (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                trigger_yaml TEXT NOT NULL,
                action_yaml TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                updated INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS scenes (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                payload TEXT NOT NULL,
                updated INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entity_id TEXT NOT NULL,
                state TEXT,
                attributes TEXT,
                timestamp INTEGER NOT NULL
            );",
        )?;
        // migration: add attributes column to entities if missing
        let _ = conn.execute(
            "ALTER TABLE entities ADD COLUMN attributes TEXT NOT NULL DEFAULT '{}'",
            [],
        );
        Ok(())
    }

    pub fn list_devices(&self) -> anyhow::Result<Vec<Device>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT id,name,kind,integration,state,updated FROM devices")?;
        let rows = stmt.query_map([], |r| {
            Ok(Device {
                id: r.get(0)?,
                name: r.get(1)?,
                kind: r.get(2)?,
                integration: r.get(3)?,
                state: r.get(4)?,
                updated: r.get(5)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn get_device(&self, id: &str) -> anyhow::Result<Option<Device>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT id,name,kind,integration,state,updated FROM devices WHERE id=?1")?;
        let res = stmt
            .query_row(params![id], |r| {
                Ok(Device {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    kind: r.get(2)?,
                    integration: r.get(3)?,
                    state: r.get(4)?,
                    updated: r.get(5)?,
                })
            })
            .optional()?;
        Ok(res)
    }

    pub fn upsert_device(&self, d: &Device) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO devices (id,name,kind,integration,state,updated) VALUES (?1,?2,?3,?4,?5,?6)
             ON CONFLICT(id) DO UPDATE SET name=excluded.name,kind=excluded.kind,integration=excluded.integration,state=excluded.state,updated=excluded.updated",
            params![d.id, d.name, d.kind, d.integration, d.state, d.updated],
        )?;
        Ok(())
    }

    pub fn delete_device(&self, id: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM devices WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn get_entity(&self, id: &str) -> anyhow::Result<Option<Entity>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id,device_id,name,kind,value,attributes,updated FROM entities WHERE id=?1")?;
        let res = stmt.query_row(rusqlite::params![id], |r| {
            Ok(Entity {
                id: r.get(0)?,
                device_id: r.get(1)?,
                name: r.get(2)?,
                kind: r.get(3)?,
                value: r.get(4)?,
                attributes: r.get::<_, Option<String>>(5)?.unwrap_or_else(|| "{}".into()),
                updated: r.get(6)?,
            })
        }).optional()?;
        Ok(res)
    }

    pub fn list_entities(&self) -> anyhow::Result<Vec<Entity>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id,device_id,name,kind,value,attributes,updated FROM entities")?;
        let rows = stmt.query_map([], |r| {
            Ok(Entity {
                id: r.get(0)?,
                device_id: r.get(1)?,
                name: r.get(2)?,
                kind: r.get(3)?,
                value: r.get(4)?,
                attributes: r.get::<_, String>(5).unwrap_or_else(|_| "{}".to_string()),
                updated: r.get(6)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn upsert_entity(&self, e: &Entity) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO entities (id,device_id,name,kind,value,attributes,updated) VALUES (?1,?2,?3,?4,?5,?6,?7)
             ON CONFLICT(id) DO UPDATE SET device_id=excluded.device_id,name=excluded.name,kind=excluded.kind,value=excluded.value,attributes=excluded.attributes,updated=excluded.updated",
            params![e.id, e.device_id, e.name, e.kind, e.value, e.attributes, e.updated],
        )?;
        Ok(())
    }

    pub fn delete_entity(&self, id: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM entities WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn record_history<E, S, A>(
        &self,
        entity_id: E,
        state: S,
        attributes: A,
    ) -> anyhow::Result<()>
    where
        E: AsRef<str>,
        S: ToNullableString,
        A: ToNullableString,
    {
        let entity_id = entity_id.as_ref();
        let state = state.to_nullable();
        let attributes = attributes.to_nullable();
        let ts = chrono::Utc::now().timestamp();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO history (entity_id, state, attributes, timestamp) VALUES (?1,?2,?3,?4)",
            params![entity_id, state, attributes, ts],
        )?;
        Ok(())
    }

    pub fn list_history(
        &self,
        entity_id: Option<&str>,
        since: i64,
    ) -> anyhow::Result<Vec<(String, String, String, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut out = Vec::new();
        if let Some(eid) = entity_id {
            let mut stmt = conn.prepare(
                "SELECT entity_id, state, attributes, timestamp FROM history WHERE entity_id=?1 AND timestamp >= ?2 ORDER BY id ASC",
            )?;
            let rows = stmt.query_map(params![eid, since], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    r.get::<_, i64>(3)?,
                ))
            })?;
            for row in rows {
                out.push(row?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT entity_id, state, attributes, timestamp FROM history WHERE timestamp >= ?1 ORDER BY id ASC",
            )?;
            let rows = stmt.query_map(params![since], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    r.get::<_, i64>(3)?,
                ))
            })?;
            for row in rows {
                out.push(row?);
            }
        }
        Ok(out)
    }

    pub fn list_automations(&self) -> anyhow::Result<Vec<Automation>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id,name,trigger_yaml,action_yaml,enabled,updated FROM automations")?;
        let rows = stmt.query_map([], |r| {
            Ok(Automation {
                id: r.get(0)?,
                name: r.get(1)?,
                trigger_yaml: r.get(2)?,
                action_yaml: r.get(3)?,
                enabled: r.get::<_, i64>(4)? != 0,
                updated: r.get(5)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn upsert_automation(&self, a: &Automation) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO automations (id,name,trigger_yaml,action_yaml,enabled,updated) VALUES (?1,?2,?3,?4,?5,?6)
             ON CONFLICT(id) DO UPDATE SET name=excluded.name,trigger_yaml=excluded.trigger_yaml,action_yaml=excluded.action_yaml,enabled=excluded.enabled,updated=excluded.updated",
            params![a.id, a.name, a.trigger_yaml, a.action_yaml, a.enabled as i64, a.updated],
        )?;
        Ok(())
    }

    pub fn delete_automation(&self, id: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM automations WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn list_scenes(&self) -> anyhow::Result<Vec<Scene>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id,name,payload,updated FROM scenes")?;
        let rows = stmt.query_map([], |r| {
            Ok(Scene {
                id: r.get(0)?,
                name: r.get(1)?,
                payload: r.get(2)?,
                updated: r.get(3)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn upsert_scene(&self, s: &Scene) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO scenes (id,name,payload,updated) VALUES (?1,?2,?3,?4)
             ON CONFLICT(id) DO UPDATE SET name=excluded.name,payload=excluded.payload,updated=excluded.updated",
            params![s.id, s.name, s.payload, s.updated],
        )?;
        Ok(())
    }

    pub fn list_groups(&self) -> anyhow::Result<Vec<Group>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id,name,members,integration,native_id,created FROM groups")?;
        let rows = stmt.query_map([], |r| {
            let members_str: String = r.get(2)?;
            let members: Vec<String> =
                serde_json::from_str(&members_str).unwrap_or_default();
            Ok(Group {
                id: r.get(0)?,
                name: r.get(1)?,
                members,
                integration: r.get(3)?,
                native_id: r.get(4)?,
                created: r.get(5)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn upsert_group(&self, g: &Group) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let members = serde_json::to_string(&g.members).unwrap_or_else(|_| "[]".to_string());
        conn.execute(
            "INSERT INTO groups (id,name,members,integration,native_id,created) VALUES (?1,?2,?3,?4,?5,?6)
             ON CONFLICT(id) DO UPDATE SET name=excluded.name,members=excluded.members,integration=excluded.integration,native_id=excluded.native_id,created=excluded.created",
            params![g.id, g.name, members, g.integration, g.native_id, g.created],
        )?;
        Ok(())
    }

    pub fn delete_group(&self, id: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM groups WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn list_links(&self) -> anyhow::Result<Vec<DeviceLink>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT id,source,target,kind,created FROM device_links")?;
        let rows = stmt.query_map([], |r| {
            Ok(DeviceLink {
                id: r.get(0)?,
                source: r.get(1)?,
                target: r.get(2)?,
                kind: r.get(3)?,
                created: r.get(4)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn upsert_link(&self, l: &DeviceLink) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO device_links (id,source,target,kind,created) VALUES (?1,?2,?3,?4,?5)
             ON CONFLICT(id) DO UPDATE SET source=excluded.source,target=excluded.target,kind=excluded.kind,created=excluded.created",
            params![l.id, l.source, l.target, l.kind, l.created],
        )?;
        Ok(())
    }

    pub fn delete_link(&self, id: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM device_links WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn list_entities_by_domain(&self, domain: &str) -> anyhow::Result<Vec<Entity>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("{}.%", domain);
        let mut stmt = conn.prepare(
            "SELECT id,device_id,name,kind,value,attributes,updated FROM entities WHERE id LIKE ?1 OR kind = ?2",
        )?;
        let rows = stmt.query_map(params![pattern, domain], |r| {
            Ok(Entity {
                id: r.get(0)?,
                device_id: r.get(1)?,
                name: r.get(2)?,
                kind: r.get(3)?,
                value: r.get(4)?,
                attributes: r.get::<_, String>(5).unwrap_or_else(|_| "{}".to_string()),
                updated: r.get(6)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }
}
