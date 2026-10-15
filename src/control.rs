//! Single-owner runtime: controls only run between committed simulation ticks.
use crate::{
    Result,
    creatures::from_life as make_entity,
    entity::{controller::EntityController, intent::Intent, phase::EntityPhase, species::Species},
    map::{cell::Cell, movement::Position},
    persistence::{Individual, State, Store},
    world::{EntitySlot, World},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    io::Read,
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Genetics {enabled:bool,mutation_per_mille:u16},
    Observation {
        full_all: bool,
        full_for: Vec<usize>,
    },
    Pause,
    Resume,
    Shutdown,
    Step {
        ticks: usize,
    },
    Speed {
        ticks_per_second: f64,
    },
    Intent {
        id: usize,
        intent: Intent,
    },
    SetCell {
        pos: Position,
        cell: Cell,
    },
    Spawn {
        species: Species,
        pos: Position,
    },
    UpdateEntity {
        entity: Individual,
    },
    Remove {
        id: usize,
    },
    SetResourceSite {
        site: crate::environment::ResourceSite,
    },
    RemoveResourceSite {
        pos: Position,
    },
}
pub struct Runtime {
    pub world: World,
    pub store: Store,
    pub paused: bool,
    pub tps: f64,
    pub shutdown: bool,
    pub error: Option<String>,
}
impl Runtime {
    pub fn status(&self) -> Value {
        json!({"alive":self.world.entities.iter().filter(|s|matches!(s.phase,EntityPhase::Active|EntityPhase::Sleeping{..}) && s.base.life().is_alive()).count(),"history_bytes":self.store.estimated_bytes().ok(),"history_budget_bytes":self.store.max_history_bytes,"tick":self.world.tick_counter,"paused":self.paused,"ticks_per_second":self.tps,"shutdown":self.shutdown,"storage_error":self.error,"shard":self.store.shard,"directory":self.store.dir,"entities":self.world.entities.len()})
    }
    pub fn tick(&mut self) -> Result<()> {
        let start = Instant::now();
        let events = self.world.tick();
        if let Err(e) = self.store.record(
            &self.world,
            &events,
            "tick",
            start.elapsed().as_micros(),
            None,
        ) {
            self.paused = true;
            self.error = Some(e.to_string());
            return Err(e);
        }
        Ok(())
    }
    pub fn apply(&mut self, value: Value) -> Result<Value> {
        if self.error.is_some() {
            return Err(
                "storage failure: restart from last durable record after repairing storage".into(),
            );
        }
        let cmd: Command = serde_json::from_value(value.clone())?;
        // Validate first; no partial intervention on invalid input.
        let result = match cmd {
            Command::Genetics{enabled,mutation_per_mille}=>{
                if mutation_per_mille>1000 {return Err("mutation rate must be 0..1000 per mille".into());}
                self.world.genetics=crate::entity::genetics::Genetics{enabled,mutation_per_mille};
                json!({"genetics":self.world.genetics})
            },
            Command::Observation { full_all, full_for } => {
                if full_for.len() > 100 {
                    return Err("at most 100 selected individuals".into());
                }
                self.world.observation =
                    crate::entity::decision::ObservationSettings { full_all, full_for };
                json!({"observation":self.world.observation})
            }
            Command::Pause => {
                self.paused = true;
                json!({"paused":true})
            }
            Command::Resume => {
                self.paused = false;
                json!({"paused":false})
            }
            Command::Shutdown => {
                self.shutdown = true;
                json!({"shutdown":true})
            }
            Command::Speed {
                ticks_per_second: n,
            } => {
                if !n.is_finite() || !(0.01..=1000.0).contains(&n) {
                    return Err("speed must be 0.01..1000 ticks/s".into());
                }
                self.tps = n;
                json!({"ticks_per_second":n})
            }
            Command::Step { ticks } => {
                if !self.paused || !(1..=100).contains(&ticks) {
                    return Err("pause first; step supports 1..100 ticks".into());
                }
                // Persist the request before the batch so interrupted batches remain identifiable.
                let started =
                    json!({"requested_ticks":ticks,"started_at_tick":self.world.tick_counter});
                if let Err(e) = self.store.record(
                    &self.world,
                    &[],
                    "step_requested",
                    0,
                    Some((&value, &started)),
                ) {
                    self.paused = true;
                    self.error = Some(e.to_string());
                    return Err(e);
                }
                for _ in 0..ticks {
                    self.tick()?;
                }
                json!({"tick":self.world.tick_counter})
            }
            Command::Intent { id, intent } => {
                let s = self
                    .world
                    .entities
                    .iter()
                    .find(|s| s.id == id)
                    .ok_or("entity not found")?;
                if !s.phase.is_active() {
                    return Err("entity is not active".into());
                }
                validate_intent(&intent)?;
                self.world.push_command(id, intent);
                json!({"queued":id})
            }
            Command::SetCell { pos, cell } => {
                self.validate_pos(pos)?;
                if matches!(cell,Cell::Food{amount}|Cell::Water{amount} if amount>1_000_000) {
                    return Err("resource amount exceeds 1000000".into());
                }
                if cell==Cell::Wall {self.validate_vacant(pos,None)?;}
                self.world.map.set_cell(pos, cell);
                json!({"updated":pos})
            }
            Command::Spawn { species, pos } => {
                self.validate_pos(pos)?;
                self.validate_vacant(pos, None)?;
                if self.world.entities.len() >= 10000 {
                    return Err("population safety limit: 10000".into());
                }
                let id = self.world.next_entity_id;
                self.world.next_entity_id += 1;
                let entity: Box<dyn crate::entity::Entity> = match species {
                    Species::Herbivore => {
                        Box::new(crate::creatures::herbivore::HerbivoreEntity::default())
                    }
                    Species::Carnivore => {
                        Box::new(crate::creatures::carnivore::CarnivoreEntity::default())
                    }
                    Species::Omnivore => {
                        Box::new(crate::creatures::omnivore::OmnivoreEntity::default())
                    }
                };
                self.world.entities.push(EntitySlot::new(
                    id,
                    pos,
                    EntityPhase::Active,
                    EntityController::AI,
                    entity,
                ));
                json!({"spawned":id})
            }
            Command::UpdateEntity { entity: e } => {
                self.validate_pos(e.pos)?;
                self.validate_vacant(e.pos, Some(e.id))?;
                e.life.validate()?;
                e.memory.validate()?;
                e.learner.validate()?;
                e.genome.validate()?;
                let s = self
                    .world
                    .entities
                    .iter_mut()
                    .find(|s| s.id == e.id)
                    .ok_or("entity not found")?;
                *s = EntitySlot::new(
                    e.id,
                    e.pos,
                    e.phase,
                    e.controller,
                    make_entity(e.species, e.life),
                );
                s.genome=e.genome;
                s.learner=e.learner;
                s.memory=e.memory;
                s.generation = e.generation;
                s.parents = e.parents;
                s.corpse_energy = e.corpse_energy;
                json!({"updated":e.id})
            }
            Command::SetResourceSite { site } => {
                self.validate_pos(site.pos)?;
                if site.capacity > 1_000_000
                    || site.replenish > site.capacity
                    || site.interval == 0
                    || site.interval > 1_000_000
                {
                    return Err("invalid resource site".into());
                }
                self.world.environment.set_site(site);
                json!({"configured":true})
            }
            Command::RemoveResourceSite { pos } => {
                self.validate_pos(pos)?;
                self.world.environment.sites.retain(|s| s.pos != pos);
                json!({"removed_source":pos})
            }
            Command::Remove { id } => {
                let s = self
                    .world
                    .entities
                    .iter_mut()
                    .find(|s| s.id == id)
                    .ok_or("entity not found")?;
                s.phase = EntityPhase::Removed;
                self.world.command_inputs.remove(&id);
                json!({"removed":id})
            }
        };
        if let Err(e) = self
            .store
            .record(&self.world, &[], "control", 0, Some((&value, &result)))
        {
            self.paused = true;
            self.error = Some(e.to_string());
            return Err(e);
        }
        Ok(result)
    }
    fn validate_pos(&self, p: Position) -> Result<()> {
        if !self.world.map.in_bounds(p) {
            Err("position out of bounds".into())
        } else {
            Ok(())
        }
    }
    fn validate_vacant(&self, p: Position, id: Option<usize>) -> Result<()> {
        if !self.world.map.is_walkable(p) {return Err("position is blocked".into());}
        if self.world.entities.iter().any(|s| {
            Some(s.id) != id
                && s.pos == p
                && matches!(s.phase, EntityPhase::Active | EntityPhase::Sleeping { .. })
        }) {
            Err("position occupied".into())
        } else {
            Ok(())
        }
    }
}
fn validate_intent(i: &Intent) -> Result<()> {
    match i {
        Intent::Move { steps } | Intent::Eat { at: steps, .. } | Intent::Drink { at: steps }
            if steps.0.len() > 32 =>
        {
            Err("path exceeds 32 steps".into())
        }
        Intent::Sleep { duration } | Intent::Idle { duration } if *duration > 1_000_000 => {
            Err("duration too large".into())
        }
        _ => Ok(()),
    }
}

pub fn serve(mut runtime: Runtime, bind: &str, token: Option<String>) -> Result<()> {
    let address: std::net::SocketAddr = bind.parse()?;
    if !address.ip().is_loopback() && token.as_deref().is_none_or(|t| t.len() < 24) {
        return Err("non-loopback bind requires CURIOUS_TOKEN with at least 24 characters".into());
    }
    let server = tiny_http::Server::http(address)?;
    eprintln!(
        "Curious control: http://{} | data: {}",
        server.server_addr(),
        runtime.store.dir.display()
    );
    let mut next = Instant::now();
    while !runtime.shutdown {
        let wait = if runtime.paused {
            Duration::from_millis(100)
        } else {
            next.saturating_duration_since(Instant::now())
                .min(Duration::from_millis(100))
        };
        if let Some(mut request) = server.recv_timeout(wait)? {
            let allowed = token.as_ref().is_none_or(|t| {
                request.headers().iter().any(|h| {
                    h.field.equiv("Authorization") && h.value.as_str() == format!("Bearer {t}")
                })
            });
            let (status, body) = if !allowed {
                (401, json!({"error":"unauthorized"}))
            } else {
                let mut bytes = Vec::new();
                let read = request
                    .as_reader()
                    .take(1024 * 1024 + 1)
                    .read_to_end(&mut bytes);
                if read.is_err() || bytes.len() > 1024 * 1024 {
                    (
                        413,
                        json!({"error":"request body exceeds 1 MiB or is unreadable"}),
                    )
                } else {
                    let method = request.method().as_str();
                    let path = request.url().split('?').next().unwrap_or("");
                    let response: Result<Value> = (|| match (method, path) {
                        ("GET", "/decisions") => {
                            Ok(serde_json::to_value(&runtime.world.last_decisions)?)
                        }
                        ("GET", "/status") => Ok(runtime.status()),
                        ("GET", "/state") => {
                            Ok(serde_json::to_value(State::capture(&runtime.world))?)
                        }
                        ("GET", "/databases") => runtime.store.inventory(),
                        ("POST", "/control") => runtime.apply(serde_json::from_slice(&bytes)?),
                        ("POST", "/query") => {
                            #[derive(Deserialize)]
                            struct Query {
                                shard: usize,
                                sql: String,
                            }
                            let q: Query = serde_json::from_slice(&bytes)?;
                            runtime.store.query(q.shard, &q.sql)
                        }
                        _ => Err(
                            "route not found; GET /status /state /databases; POST /control /query"
                                .into(),
                        ),
                    })();
                    match response {
                        Ok(v) => (200, v),
                        Err(e) => (400, json!({"error":e.to_string()})),
                    }
                }
            };
            let response = tiny_http::Response::from_string(body.to_string())
                .with_status_code(status)
                .with_header(
                    tiny_http::Header::from_bytes("Content-Type", "application/json").unwrap(),
                );
            let _ = request.respond(response);
        }
        if !runtime.paused && !runtime.shutdown && Instant::now() >= next {
            if let Err(e) = runtime.tick() {
                eprintln!("Simulation paused: {e}");
            }
            next = Instant::now() + Duration::from_secs_f64(1.0 / runtime.tps);
        }
    }
    Ok(())
}

pub fn run_cli() -> Result<()> {
    let mut bind = "127.0.0.1:7878".to_string();
    let mut dir: Option<PathBuf> = None;
    let mut resume = false;
    let mut resume_or_new = false;
    let mut history_mb = 10240u64;
    let mut paused = false;
    let mut seed = 12345;
    let mut scenario: Option<PathBuf> = None;
    let mut tps = 3.0;
    let mut max_mb = 256u64;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--serve" => {}
            "--paused" => paused = true,
            "--resume" => resume = true,
            "--resume-or-new" => resume_or_new = true,
            "--bind" => bind = args.next().ok_or("missing --bind value")?,
            "--data-dir" => dir = Some(args.next().ok_or("missing --data-dir value")?.into()),
            "--scenario" => scenario = Some(args.next().ok_or("missing --scenario value")?.into()),
            "--seed" => seed = args.next().ok_or("missing --seed value")?.parse()?,
            "--tps" => tps = args.next().ok_or("missing --tps value")?.parse::<f64>()?,
            "--max-history-mb" => {
                history_mb = args
                    .next()
                    .ok_or("missing --max-history-mb value")?
                    .parse()?
            }
            "--shard-mb" => max_mb = args.next().ok_or("missing --shard-mb value")?.parse()?,
            _ => return Err(format!("unknown option: {arg}").into()),
        }
    }
    if !tps.is_finite() || !(0.01..=1000.0).contains(&tps) || !(1..=65536).contains(&max_mb) {
        return Err("tps: 0.01..1000; shard-mb: 1..65536".into());
    }
    if (resume || resume_or_new) && dir.is_none() {
        return Err("--resume requires --data-dir".into());
    }
    let dir =
        dir.unwrap_or_else(|| PathBuf::from(format!("data/run-{}", crate::persistence::now_ms())));
    // Separate processes must never write the same run directory.
    std::fs::create_dir_all(&dir)?;
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.join("runtime.lock"))?;
    lock.try_lock()?;
    let mut store = Store::open(&dir, max_mb * 1024 * 1024)?;
    if history_mb < max_mb || history_mb > 1_048_576 {
        return Err("max-history-mb must be >= shard-mb and <= 1048576".into());
    }
    store.max_history_bytes = history_mb * 1024 * 1024;
    let previous = store.latest()?;
    if resume_or_new && previous.is_some() {
        resume = true;
    }
    let world = if resume {
        previous.ok_or("no checkpoint found")?.restore()?
    } else {
        if previous.is_some() {
            return Err("directory already contains a run; use --resume or a new directory".into());
        }
        if let Some(path) = scenario {
            let scenario: crate::scenario::Scenario =
                serde_json::from_slice(&std::fs::read(path)?)?;
            scenario.build(seed)?
        } else {
            let entities = vec![
                EntitySlot::new(
                    1,
                    Position::new(-5, -5),
                    EntityPhase::Active,
                    EntityController::AI,
                    Box::new(crate::creatures::herbivore::HerbivoreEntity::default()),
                ),
                EntitySlot::new(
                    2,
                    Position::new(-4, -5),
                    EntityPhase::Active,
                    EntityController::AI,
                    Box::new(crate::creatures::herbivore::HerbivoreEntity::default()),
                ),
                EntitySlot::new(
                    3,
                    Position::new(5, 5),
                    EntityPhase::Active,
                    EntityController::AI,
                    Box::new(crate::creatures::carnivore::CarnivoreEntity::default()),
                ),
                EntitySlot::new(
                    4,
                    Position::new(4, 5),
                    EntityPhase::Active,
                    EntityController::AI,
                    Box::new(crate::creatures::omnivore::OmnivoreEntity::default()),
                ),
            ];
            let mut w = World::with_seed(-15, 14, -15, 14, entities, seed);
            w.logger = crate::logger::Logger::disabled();
            w
        }
    };
    store.record(
        &world,
        &[],
        if resume { "resume" } else { "initial" },
        0,
        None,
    )?;
    let runtime = Runtime {
        world,
        store,
        paused,
        tps,
        shutdown: false,
        error: None,
    };
    let result = serve(runtime, &bind, std::env::var("CURIOUS_TOKEN").ok());
    drop(lock);
    result
}
