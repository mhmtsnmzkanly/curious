//! Versioned observations and size-rotated SQLite history. No automatic history deletion.
use crate::{
    entity::{
        Entity, controller::EntityController, intent::Intent, lifestate::LifeState,
        phase::EntityPhase, species::Species,
    },
    map::{Map, cell::Cell, movement::Position},
    simulation::SimulationEvent,
    world::{EntitySlot, World},
};
use rusqlite::{Connection, OpenFlags, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Individual {
    pub id: usize,
    pub pos: Position,
    pub phase: EntityPhase,
    pub controller: EntityController,
    pub species: Species,
    pub life: LifeState,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub version: u32,
    pub tick: usize,
    pub rng: u64,
    pub next_entity_id: usize,
    pub bounds: [isize; 4],
    pub entities: Vec<Individual>,
    pub resources: Vec<(Position, Cell)>,
    pub commands: Vec<(usize, Intent)>,
}
impl State {
    pub fn capture(world: &World) -> Self {
        let mut resources = Vec::new();
        for y in world.map.min_y..=world.map.max_y {
            for x in world.map.min_x..=world.map.max_x {
                let p = Position::new(x, y);
                if let Some(cell @ (Cell::Food { .. } | Cell::Water { .. })) = world.map.cell(p) {
                    resources.push((p, cell.clone()));
                }
            }
        }
        let mut commands: Vec<_> = world
            .command_inputs
            .iter()
            .map(|(id, i)| (*id, i.clone()))
            .collect();
        commands.sort_by_key(|(id, _)| *id);
        Self {
            version: 1,
            tick: world.tick_counter,
            rng: world.rng.state(),
            next_entity_id: world.next_entity_id,
            bounds: [
                world.map.min_x,
                world.map.max_x,
                world.map.min_y,
                world.map.max_y,
            ],
            resources,
            commands,
            entities: world
                .entities
                .iter()
                .map(|s| Individual {
                    id: s.id,
                    pos: s.pos,
                    phase: s.phase,
                    controller: s.controller.clone(),
                    species: s.base.species(),
                    life: s.base.life().clone(),
                })
                .collect(),
        }
    }
    pub fn restore(self) -> Result<World> {
        if self.version != 1 {
            return Err("unsupported checkpoint version".into());
        }
        let [a, b, c, d] = self.bounds;
        let mut map = Map::new(a, b, c, d);
        for (p, cell) in self.resources {
            map.set_cell(p, cell);
        }
        let entities = self
            .entities
            .into_iter()
            .map(|s| {
                EntitySlot::new(
                    s.id,
                    s.pos,
                    s.phase,
                    s.controller,
                    make_entity(s.species, s.life),
                )
            })
            .collect();
        Ok(World {
            rng: crate::random::Random::new(self.rng),
            map,
            entities,
            tick_counter: self.tick,
            logger: crate::logger::Logger::disabled(),
            command_inputs: self.commands.into_iter().collect(),
            last_trace: Vec::new(),
            last_intents: Vec::new(),
            next_entity_id: self.next_entity_id,
        })
    }
}
pub fn make_entity(species: Species, life: LifeState) -> Box<dyn Entity> {
    let mut entity: Box<dyn Entity> = match species {
        Species::Herbivore => Box::new(crate::creatures::herbivore::HerbivoreEntity::default()),
        Species::Carnivore => Box::new(crate::creatures::carnivore::CarnivoreEntity::default()),
        Species::Omnivore => Box::new(crate::creatures::omnivore::OmnivoreEntity::default()),
    };
    *entity.life_mut() = life;
    entity
}
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
        let metrics = json!({"alive":alive,"entities":state.entities.len(),"resources":state.resources.len(),"food":food,"water":water,"tick_us":elapsed_us,
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
