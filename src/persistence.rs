//! Size-rotated SQLite history; engine checkpoints do not depend on this module.
pub use crate::Result;
pub use crate::simulation::checkpoint::{Individual, State};
use crate::{
    entity::phase::EntityPhase, map::cell::Cell, simulation::SimulationEvent, world::World,
};
use rusqlite::{Connection, params};
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
        crate::archive::cleanup_pending(dir)?;
        let shard = Self::files(dir)?.last().map(|(n, _)| *n).unwrap_or(0);
        // A run whose last available file is sealed gets a fresh active shard.
        let shard = if Self::files(dir)?
            .last()
            .is_some_and(|(_, p)| crate::archive::is_archive(p))
        {
            shard + 1
        } else {
            shard
        };
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
        let mut files = std::collections::BTreeMap::<usize, PathBuf>::new();
        for entry in std::fs::read_dir(dir)? {
            let p = entry?.path();
            let Some(name) = p.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            let raw = name.strip_suffix(".gz").unwrap_or(name);
            let Some(n) = raw
                .strip_prefix("history-")
                .and_then(|s| s.strip_suffix(".sqlite"))
                .and_then(|s| s.parse().ok())
            else {
                continue;
            };
            if !crate::archive::is_archive(&p) || !files.contains_key(&n) {
                files.insert(n, p);
            }
        }
        Ok(files.into_iter().collect())
    }
    fn connect(path: &Path) -> Result<Connection> {
        let c = Connection::open(path)?;
        let version: i64 = c.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version > 2 {
            return Err("unsupported history schema version".into());
        }
        c.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; BEGIN IMMEDIATE;
          CREATE TABLE IF NOT EXISTS records(id INTEGER PRIMARY KEY, tick INTEGER NOT NULL, at_ms TEXT NOT NULL, kind TEXT NOT NULL, state_json TEXT NOT NULL, metrics_json TEXT NOT NULL, trace_json TEXT NOT NULL);
          CREATE INDEX IF NOT EXISTS records_tick ON records(tick);
          CREATE TABLE IF NOT EXISTS events(record_id INTEGER NOT NULL, ordinal INTEGER NOT NULL, kind TEXT NOT NULL, payload TEXT NOT NULL, PRIMARY KEY(record_id,ordinal));
          CREATE INDEX IF NOT EXISTS events_kind ON events(kind);
          CREATE TABLE IF NOT EXISTS decisions(record_id INTEGER NOT NULL, entity_id INTEGER NOT NULL, intent_json TEXT NOT NULL);
          CREATE INDEX IF NOT EXISTS decisions_entity ON decisions(entity_id,record_id);
          CREATE TABLE IF NOT EXISTS decision_traces(record_id INTEGER NOT NULL,entity_id INTEGER NOT NULL,trace_json TEXT NOT NULL,PRIMARY KEY(record_id,entity_id));
          CREATE INDEX IF NOT EXISTS traces_entity ON decision_traces(entity_id,record_id);
          CREATE TABLE IF NOT EXISTS interventions(id INTEGER PRIMARY KEY, tick INTEGER NOT NULL, at_ms TEXT NOT NULL, request_json TEXT NOT NULL, result_json TEXT NOT NULL); PRAGMA user_version=2; COMMIT;")?;
        Ok(c)
    }
    fn rotate(&mut self) -> Result<()> {
        let pages: i64 = self.conn.query_row("PRAGMA page_count", [], |r| r.get(0))?;
        let size: i64 = self.conn.query_row("PRAGMA page_size", [], |r| r.get(0))?;
        if self.archived_bytes + (pages * size) as u64 >= self.max_history_bytes {
            return Err("history storage budget reached; archive old shards and restart with a larger budget".into());
        }
        if (pages * size) as u64 >= self.max_bytes {
            let (busy, frames, checkpointed): (i64, i64, i64) =
                self.conn
                    .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |r| {
                        Ok((r.get(0)?, r.get(1)?, r.get(2)?))
                    })?;
            if busy != 0 || frames > checkpointed {
                return Err("cannot seal history while WAL checkpoint is busy".into());
            }
            let next = self.shard + 1;
            let conn = Self::connect(&self.dir.join(format!("history-{next:06}.sqlite")))?;
            let previous = self.dir.join(format!("history-{:06}.sqlite", self.shard));
            let old = std::mem::replace(&mut self.conn, conn);
            self.shard = next;
            drop(old);
            let archived = crate::archive::seal(&previous)?;
            self.archived_bytes += std::fs::metadata(archived)?.len();
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
            for trace in &world.last_decisions {
                tx.execute(
                    "INSERT INTO decision_traces VALUES(?1,?2,?3)",
                    params![id, trace.id as i64, serde_json::to_string(trace)?],
                )?;
            }
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
            let c = crate::archive::Reader::open(&path)?;
            let mut stmt = c.prepare("SELECT state_json FROM records ORDER BY id DESC LIMIT 1")?;
            let mut rows = stmt.query([])?;
            if let Some(row) = rows.next()? {
                let s: String = row.get(0)?;
                return Ok(Some(serde_json::from_str(&s)?));
            }
        }
        Ok(None)
    }
    pub fn estimated_bytes(&self) -> Result<u64> {
        let pages: i64 = self.conn.query_row("PRAGMA page_count", [], |r| r.get(0))?;
        let size: i64 = self.conn.query_row("PRAGMA page_size", [], |r| r.get(0))?;
        Ok(self.archived_bytes + (pages * size) as u64)
    }
    pub fn inventory(&self) -> Result<Value> {
        Ok(json!(Self::files(&self.dir)?.into_iter().map(|(n,p)|json!({"shard":n,"compressed":crate::archive::is_archive(&p),"bytes":std::fs::metadata(p).map(|m|m.len()).unwrap_or(0)})).collect::<Vec<_>>()))
    }
    /// Read-only bounded inspection on any historical shard; no SQL writes or attachments.
    pub fn query(&self, shard: usize, sql: &str) -> Result<Value> {
        let path = Self::files(&self.dir)?
            .into_iter()
            .find(|(n, _)| *n == shard)
            .map(|(_, p)| p)
            .ok_or("history shard not found")?;
        let c = crate::archive::Reader::open(&path)?;
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
