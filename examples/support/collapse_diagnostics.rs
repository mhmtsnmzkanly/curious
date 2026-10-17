//! Optional observer: never changes simulation state or consumes RNG.
use curious::{
    entity::species::Species,
    map::cell::Cell,
    simulation::{SimulationEvent, checkpoint::State},
    world::World,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, VecDeque},
    fs::{self, File},
    io::Write,
    path::PathBuf,
};
pub struct Diagnostics {
    from: usize,
    dir: PathBuf,
    file: File,
    counts: BTreeMap<String, usize>,
    recent: BTreeMap<usize, Value>,
    deaths: VecDeque<Value>,
}
impl Diagnostics {
    pub fn new(dir: &str, from: usize, seed: u64) -> Self {
        let dir = PathBuf::from(dir).join(format!("seed-{seed}"));
        fs::create_dir(&dir).expect("diagnostics requires a fresh seed directory");
        let file = File::create(dir.join("windows.jsonl")).unwrap();
        Self {
            from,
            dir,
            file,
            counts: BTreeMap::new(),
            recent: BTreeMap::new(),
            deaths: VecDeque::new(),
        }
    }
    pub fn observe(&mut self, w: &World, events: &[SimulationEvent]) {
        let tick = w.tick_counter;
        if tick < self.from {
            return;
        }
        let mut index = BTreeMap::new();
        for e in &w.entities {
            index.insert(e.id, e);
        }
        for t in &w.last_decisions {
            *self
                .counts
                .entry(format!("{:?}.decision.{:?}", t.species, t.decision.reason))
                .or_default() += 1;
            if t.species == Species::Herbivore {
                *self
                    .counts
                    .entry(format!(
                        "Herbivore.blocker.{:?}",
                        t.before.reproduction_blocker()
                    ))
                    .or_default() += 1;
                self.recent.insert(t.id,json!({"tick":tick,"pos":t.pos,"life":t.before,"reason":t.decision.reason,"seen":t.seen,"outcome":t.outcome}));
            }
        }
        for event in events {
            match event {
                SimulationEvent::EntityDied {
                    id, species, cause, ..
                } => {
                    *self
                        .counts
                        .entry(format!("{species:?}.death.{cause:?}"))
                        .or_default() += 1;
                    if *species == Species::Herbivore {
                        let e = index[id];
                        self.deaths.push_back(json!({"tick":tick,"id":id,"cause":cause,"pos":e.pos,"life":e.base.life(),"parents":e.parents,"last_decision":self.recent.remove(id)}));
                        if self.deaths.len() > 64 {
                            self.deaths.pop_front();
                        }
                    }
                }
                SimulationEvent::EntityBorn { species, .. } => {
                    *self.counts.entry(format!("{species:?}.birth")).or_default() += 1
                }
                SimulationEvent::ActionRejected {
                    id, action, reason, ..
                } => {
                    if let Some(e) = index.get(id) {
                        *self
                            .counts
                            .entry(format!("{:?}.reject.{action}.{reason:?}", e.base.species()))
                            .or_default() += 1;
                    }
                }
                SimulationEvent::EntityAte { id, amount, .. } => {
                    if let Some(e) = index.get(id) {
                        *self
                            .counts
                            .entry(format!("{:?}.food_intake", e.base.species()))
                            .or_default() += amount;
                    }
                }
                _ => {}
            }
        }
        let alive: Vec<_> = w
            .entities
            .iter()
            .filter(|e| e.base.life().health > 0)
            .collect();
        let herbs: Vec<_> = alive
            .iter()
            .filter(|e| e.base.species() == Species::Herbivore)
            .collect();
        if tick % 100 == 0 || herbs.is_empty() {
            let herb_state:Vec<_>=herbs.iter().map(|e|json!({"id":e.id,"pos":e.pos,"life":e.base.life(),"sheltered":matches!(w.map.cell(e.pos),Some(Cell::Burrow)),"last_decision":self.recent.get(&e.id)})).collect();
            let mut totals = BTreeMap::<String, usize>::new();
            for e in &alive {
                *totals.entry(format!("{:?}", e.base.species())).or_default() += 1;
            }
            let food: usize = State::capture(w)
                .resources
                .iter()
                .map(|(_, c)| c)
                .filter_map(|c| {
                    if let Cell::Food { amount } = c {
                        Some(*amount)
                    } else {
                        None
                    }
                })
                .sum();
            writeln!(self.file,"{}",json!({"tick":tick,"counts":self.counts,"alive":totals,"plant_stock":food,"herbivores":herb_state})).unwrap();
            self.file.flush().unwrap();
            self.counts.clear();
        }
        if [16000, 16800, 17000, 17400, 17600].contains(&tick) {
            fs::write(
                self.dir.join(format!("checkpoint-{tick}.json")),
                serde_json::to_vec(&State::capture(w)).unwrap(),
            )
            .unwrap();
        }
        if herbs.is_empty() {
            fs::write(
                self.dir.join("last-herbivore-deaths.json"),
                serde_json::to_vec_pretty(&self.deaths).unwrap(),
            )
            .unwrap();
        }
        self.recent
            .retain(|id, _| index.get(id).is_some_and(|e| e.base.life().health > 0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use curious::scenario::Scenario;
    #[test]
    fn observer_preserves_every_tick_state_and_rng() {
        let scenario = include_str!("../../scenarios/meadow.json");
        let make = || {
            serde_json::from_str::<Scenario>(scenario)
                .unwrap()
                .build(2)
                .unwrap()
        };
        let (mut plain, mut observed) = (make(), make());
        let dir = tempfile::tempdir().unwrap();
        let mut observer = Diagnostics::new(dir.path().to_str().unwrap(), 1, 2);
        for _ in 0..250 {
            let expected = plain.tick();
            let actual = observed.tick();
            observer.observe(&observed, &actual);
            assert_eq!(
                serde_json::to_value(expected).unwrap(),
                serde_json::to_value(actual).unwrap()
            );
            assert_eq!(
                serde_json::to_value(State::capture(&plain)).unwrap(),
                serde_json::to_value(State::capture(&observed)).unwrap()
            );
        }
        assert!(
            dir.path()
                .join("seed-2/windows.jsonl")
                .metadata()
                .unwrap()
                .len()
                > 0
        );
    }
}
