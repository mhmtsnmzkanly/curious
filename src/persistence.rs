//! Size-rotated SQLite history; engine checkpoints do not depend on this module.
pub use crate::Result;
pub use crate::simulation::checkpoint::{Individual, State};
use crate::{
    entity::phase::EntityPhase, map::cell::Cell, simulation::SimulationEvent, world::World,
};
use rusqlite::{Connection, OpenFlags, params};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

pub struct Store {
    pub dir: PathBuf,
    pub shard: usize,
    conn: Connection,
    max_bytes: u64,
    pub max_history_bytes: u64,
    archived_bytes: u64,
}
impl Store {
    pub fn open(dir: &Path, max_bytes: u64) -> Result<Self> {
        std::fs::create_dir_all(dir)?;
        let shard = Self::files(dir)?.last().map(|(n, _)| *n).unwrap_or(0);
        let conn = Self::connect(&dir.join(format!("history-{shard:06}.sqlite")))?;
        let archived_bytes = Self::files(dir)?
            .into_iter()
            .filter(|(n, _)| *n != shard)
            .map(|(_, p)| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0))
            .sum();
        Ok(Self {
            dir: dir.into(),
            shard,
            conn,
            max_bytes,
            max_history_bytes: 10 * 1024 * 1024 * 1024,
            archived_bytes,
        })
    }
    fn files(dir: &Path) -> Result<Vec<(usize, PathBuf)>> {
        let mut files = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let p = entry?.path();
            if let Some(n) = p
                .file_name()
                .and_then(|s| s.to_str())
                .and_then(|s| s.strip_prefix("history-"))
                .and_then(|s| s.strip_suffix(".sqlite"))
                .and_then(|s| s.parse().ok())
            {
                files.push((n, p));
            }
        }
        files.sort_by_key(|(n, _)| *n);
        Ok(files)
    }
    fn connect(path: &Path) -> Result<Connection> {
        let c = Connection::open(path)?;
        c.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA user_version=1;
          CREATE TABLE IF NOT EXISTS records(id INTEGER PRIMARY KEY, tick INTEGER NOT NULL, at_ms TEXT NOT NULL, kind TEXT NOT NULL, state_json TEXT NOT NULL, metrics_json TEXT NOT NULL, trace_json TEXT NOT NULL);
          CREATE INDEX IF NOT EXISTS records_tick ON records(tick);
          CREATE TABLE IF NOT EXISTS events(record_id INTEGER NOT NULL, ordinal INTEGER NOT NULL, kind TEXT NOT NULL, payload TEXT NOT NULL, PRIMARY KEY(record_id,ordinal));
          CREATE INDEX IF NOT EXISTS events_kind ON events(kind);
          CREATE TABLE IF NOT EXISTS decisions(record_id INTEGER NOT NULL, entity_id INTEGER NOT NULL, intent_json TEXT NOT NULL);
          CREATE INDEX IF NOT EXISTS decisions_entity ON decisions(entity_id,record_id);
          CREATE TABLE IF NOT EXISTS interventions(id INTEGER PRIMARY KEY, tick INTEGER NOT NULL, at_ms TEXT NOT NULL, request_json TEXT NOT NULL, result_json TEXT NOT NULL);")?;
        Ok(c)
    }
    fn rotate(&mut self) -> Result<()> {
        let pages: i64 = self.conn.query_row("PRAGMA page_count", [], |r| r.get(0))?;
        let size: i64 = self.conn.query_row("PRAGMA page_size", [], |r| r.get(0))?;
        if self.archived_bytes + (pages * size) as u64 >= self.max_history_bytes {
            return Err("history storage budget reached; archive old shards and restart with a larger budget".into());
        }
        if (pages * size) as u64 >= self.max_bytes {
            self.conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")?;
            let next = self.shard + 1;
            let conn = Self::connect(&self.dir.join(format!("history-{next:06}.sqlite")))?;
            self.archived_bytes += (pages * size) as u64;
            self.conn = conn;
            self.shard = next;
        }
        Ok(())
    }
    pub fn record(
        &mut self,
        world: &World,
        events: &[SimulationEvent],
        kind: &str,
        elapsed_us: u128,
        intervention: Option<(&Value, &Value)>,
    ) -> Result<()> {
        self.rotate()?;
        let state = State::capture(world);
        let alive = state
            .entities
            .iter()
            .filter(|e| matches!(e.phase, EntityPhase::Active | EntityPhase::Sleeping { .. }))
            .count();
        let food: usize = state
            .resources
            .iter()
            .map(|(_, c)| {
                if let Cell::Food { amount } = c {
                    *amount
                } else {
                    0
                }
            })
            .sum();
        let water: usize = state
            .resources
            .iter()
            .map(|(_, c)| {
                if let Cell::Water { amount } = c {
                    *amount
                } else {
                    0
                }
            })
            .sum();
        let mut reproduction_blockers = std::collections::BTreeMap::<String, usize>::new();
        let mut by_species = std::collections::BTreeMap::<String, usize>::new();
        let mut max_generation = 0;
        for e in state
            .entities
            .iter()
            .filter(|e| matches!(e.phase, EntityPhase::Active | EntityPhase::Sleeping { .. }))
        {
            *by_species.entry(format!("{:?}", e.species)).or_default() += 1;
            let reason = e
                .life
                .reproduction_blocker()
                .map(|r| format!("{r:?}"))
                .unwrap_or_else(|| "Ready".into());
            *reproduction_blockers.entry(reason).or_default() += 1;
            max_generation = max_generation.max(e.generation);
        }
        let metrics = json!({"alive":alive,"by_species":by_species,"max_living_generation":max_generation,"reproduction_blockers":reproduction_blockers,"entities":state.entities.len(),"resources":state.resources.len(),"food":food,"water":water,"tick_us":elapsed_us,
            "births":events.iter().filter(|e|matches!(e,SimulationEvent::EntityBorn{..})).count(),
            "deaths":events.iter().filter(|e|matches!(e,SimulationEvent::EntityDied{..})).count()});
        let tx = self.conn.transaction()?;
        tx.execute("INSERT INTO records(tick,at_ms,kind,state_json,metrics_json,trace_json) VALUES(?1,?2,?3,?4,?5,?6)",params![state.tick as i64,now_ms().to_string(),kind,serde_json::to_string(&state)?,metrics.to_string(),serde_json::to_string(&world.last_trace)?])?;
        let id = tx.last_insert_rowid();
        for (n, event) in events.iter().enumerate() {
            let value = serde_json::to_value(event)?;
            let kind = value
                .as_object()
                .and_then(|o| o.keys().next())
                .cloned()
                .unwrap_or_default();
            tx.execute(
                "INSERT INTO events VALUES(?1,?2,?3,?4)",
                params![id, n as i64, kind, value.to_string()],
            )?;
        }
        if kind == "tick" {
            for (entity, intent) in &world.last_intents {
                tx.execute(
                    "INSERT INTO decisions VALUES(?1,?2,?3)",
                    params![id, *entity as i64, serde_json::to_string(intent)?],
                )?;
            }
        }
        if let Some((request, result)) = intervention {
            tx.execute("INSERT INTO interventions(tick,at_ms,request_json,result_json) VALUES(?1,?2,?3,?4)",params![state.tick as i64,now_ms().to_string(),request.to_string(),result.to_string()])?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn latest(&self) -> Result<Option<State>> {
        for (_, path) in Self::files(&self.dir)?.into_iter().rev() {
            let c = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
            let mut stmt = c.prepare("SELECT state_json FROM records ORDER BY id DESC LIMIT 1")?;
            let mut rows = stmt.query([])?;
            if let Some(row) = rows.next()? {
                let s: String = row.get(0)?;
                return Ok(Some(serde_json::from_str(&s)?));
            }
        }
        Ok(None)
    }
    pub fn inventory(&self) -> Result<Value> {
        Ok(json!(Self::files(&self.dir)?.into_iter().map(|(n,p)|json!({"shard":n,"bytes":std::fs::metadata(p).map(|m|m.len()).unwrap_or(0)})).collect::<Vec<_>>()))
    }
    /// Read-only bounded inspection on any historical shard; no SQL writes or attachments.
    pub fn query(&self, shard: usize, sql: &str) -> Result<Value> {
        let path = self.dir.join(format!("history-{shard:06}.sqlite"));
        let c = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        c.execute_batch("PRAGMA query_only=ON")?;
        let started = std::time::Instant::now();
        c.progress_handler(1000, Some(move || started.elapsed().as_millis() > 200))?;
        let upper = sql.trim().to_ascii_uppercase();
        if !(upper.starts_with("SELECT ") || upper.starts_with("WITH ")) {
            return Err("only SELECT / WITH queries supported".into());
        }
        let mut stmt = c.prepare(sql)?;
        if !stmt.readonly() {
            return Err("read-only queries required".into());
        }
        let names = stmt
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let mut cursor = stmt.query([])?;
        let mut output = Vec::new();
        let mut bytes = 0;
        while let Some(row) = cursor.next()? {
            let mut values = Vec::new();
            for i in 0..names.len() {
                use rusqlite::types::ValueRef;
                values.push(match row.get_ref(i)? {
                    ValueRef::Null => Value::Null,
                    ValueRef::Integer(n) => json!(n),
                    ValueRef::Real(n) => json!(n),
                    ValueRef::Text(t) => json!(String::from_utf8_lossy(t)),
                    ValueRef::Blob(b) => json!({"blob_bytes":b.len()}),
                });
            }
            bytes += serde_json::to_vec(&values)?.len();
            if bytes > 4 * 1024 * 1024 || output.len() >= 1000 {
                return Ok(json!({"columns":names,"rows":output,"truncated":true}));
            }
            output.push(values);
        }
        Ok(json!({"columns":names,"rows":output,"truncated":false}))
    }
}
