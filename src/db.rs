use crate::domain::{Automation, Device, Entity, Scene};
use rusqlite::{params, Connection, OptionalExtension};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

impl Db {
    pub fn open(data_dir: &str) -> anyhow::Result<Self> {
        std::fs::create_dir_all(data_dir)?;
        let path = std::path::Path::new(data_dir).join("irondom.db");
        let conn = Connection::open(path)?;
        let db = Self { conn: Arc::new(Mutex::new(conn)) };
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
            );",
        )?;
        Ok(())
    }

    pub fn list_devices(&self) -> anyhow::Result<Vec<Device>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id,name,kind,integration,state,updated FROM devices")?;
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
        let mut stmt = conn.prepare("SELECT id,name,kind,integration,state,updated FROM devices WHERE id=?1")?;
        let res = stmt.query_row(params![id], |r| {
            Ok(Device {
                id: r.get(0)?,
                name: r.get(1)?,
                kind: r.get(2)?,
                integration: r.get(3)?,
                state: r.get(4)?,
                updated: r.get(5)?,
            })
        }).optional()?;
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

    pub fn list_entities(&self) -> anyhow::Result<Vec<Entity>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id,device_id,name,kind,value,updated FROM entities")?;
        let rows = stmt.query_map([], |r| {
            Ok(Entity {
                id: r.get(0)?,
                device_id: r.get(1)?,
                name: r.get(2)?,
                kind: r.get(3)?,
                value: r.get(4)?,
                updated: r.get(5)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows { out.push(row?); }
        Ok(out)
    }

    pub fn upsert_entity(&self, e: &Entity) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO entities (id,device_id,name,kind,value,updated) VALUES (?1,?2,?3,?4,?5,?6)
             ON CONFLICT(id) DO UPDATE SET device_id=excluded.device_id,name=excluded.name,kind=excluded.kind,value=excluded.value,updated=excluded.updated",
            params![e.id, e.device_id, e.name, e.kind, e.value, e.updated],
        )?;
        Ok(())
    }

    pub fn list_automations(&self) -> anyhow::Result<Vec<Automation>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id,name,trigger_yaml,action_yaml,enabled,updated FROM automations")?;
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
        for row in rows { out.push(row?); }
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
        for row in rows { out.push(row?); }
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
}
