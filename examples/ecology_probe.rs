//! Fast deterministic ecology diagnostics; does not touch any runtime/history.
use curious::{scenario::Scenario, simulation::SimulationEvent};
use std::{
    collections::{BTreeMap, VecDeque},
    env, fs,
    time::Instant,
};
#[path = "support/collapse_diagnostics.rs"]
mod collapse_diagnostics;
fn main() {
    let args: Vec<_> = env::args().collect();
    let file = &args[1];
    let ticks: usize = args[2].parse().unwrap();
    let setting = |name: &str, default: usize| {
        args.iter()
            .find_map(|a| a.strip_prefix(name))
            .map(|v| v.parse::<usize>().unwrap())
            .unwrap_or(default)
    };
    let sample_every = setting("--sample-every=", 100);
    let shock_at = setting("--shock-at=", 0);
    let shock_duration = setting("--shock-duration=", 500);
    let shock_percent = setting("--shock-percent=", 50);
    let progress_every = setting("--progress-every=", 0);
    assert!(sample_every > 0 && shock_duration > 0 && shock_percent <= 100);
    for seed in args[3].split(',').map(|s| s.parse::<u64>().unwrap()) {
        let mut w = serde_json::from_str::<Scenario>(&fs::read_to_string(file).unwrap())
            .unwrap()
            .build(seed)
            .unwrap();
        w.policy.mate_rendezvous = args.iter().any(|a| a == "--mate-rendezvous");
        w.policy.reserve_opportunities = args.iter().any(|a| a == "--reserve-opportunities");
        let mode = args.get(4).map(String::as_str).unwrap_or("both");
        let stop_early = args.iter().any(|a| a == "--stop-on-extinction");
        let companions = !args.iter().any(|a| a == "--no-companions");
        assert!(["fixed", "learning", "mutation", "both"].contains(&mode));
        w.genetics.enabled = matches!(mode, "mutation" | "both");
        for e in &mut w.entities {
            e.learner.enabled = matches!(mode, "learning" | "both");
        }
        let mut births: BTreeMap<String, usize> = BTreeMap::new();
        let mut breeders = BTreeMap::new();
        let mut generations = BTreeMap::new();
        let mut last_alive = BTreeMap::new();
        let mut late_births: BTreeMap<String, usize> = BTreeMap::new();
        let mut late_breeders = BTreeMap::new();
        let mut first_extinction = None;
        let original_sites = w.environment.sites.clone();
        let mut samples = Vec::new();
        let mut death_causes = BTreeMap::<String, BTreeMap<String, usize>>::new();
        let mut predator_reasons = BTreeMap::<String, usize>::new();
        let mut predator_ready = 0usize;
        let mut predator_ready_with_mate = 0usize;
        let mut recent = BTreeMap::<usize, VecDeque<serde_json::Value>>::new();
        let mut last_predator_deaths = VecDeque::new();
        let mut diagnostics = args
            .iter()
            .find_map(|a| a.strip_prefix("--diagnostic-dir="))
            .map(|dir| {
                collapse_diagnostics::Diagnostics::new(
                    dir,
                    setting("--diagnostic-from=", 16000),
                    seed,
                )
            });
        // Observer-only checkpoints for short, reproducible failure branches.
        let checkpoint_dir = args
            .iter()
            .find_map(|a| a.strip_prefix("--checkpoint-dir="));
        let checkpoint_ticks: Vec<usize> = args
            .iter()
            .find_map(|a| a.strip_prefix("--checkpoint-at="))
            .map(|v| v.split(',').map(|n| n.parse().unwrap()).collect())
            .unwrap_or_default();
        let checkpoint_dir = checkpoint_dir.map(|root| {
            let dir = std::path::Path::new(root).join(format!("seed-{seed}"));
            fs::create_dir(&dir).expect("checkpoint directory must be fresh");
            dir
        });
        let start = Instant::now();
        for _ in 0..ticks {
            // Explicit laboratory intervention, never an automatic population repair.
            if shock_at > 0 && w.tick_counter + 1 == shock_at {
                for site in &mut w.environment.sites {
                    site.replenish = site.replenish * (100 - shock_percent) / 100;
                }
            }
            if shock_at > 0 && w.tick_counter + 1 == shock_at + shock_duration {
                w.environment.sites = original_sites.clone();
            }
            // Offspring start with their own defaults; the experiment's learning
            // treatment applies to newborns as well as founders, before decisions.
            if matches!(mode, "fixed" | "mutation") {
                for e in &mut w.entities {
                    e.learner.enabled = false;
                }
            }
            for e in &mut w.entities {
                e.memory.companions_enabled = companions;
            }
            let events = w.tick();
            if checkpoint_ticks.contains(&w.tick_counter) {
                if let Some(dir) = &checkpoint_dir {
                    use std::io::Write;
                    let mut out = fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(dir.join(format!("checkpoint-{}.json", w.tick_counter)))
                        .unwrap();
                    out.write_all(
                        &serde_json::to_vec(&curious::simulation::checkpoint::State::capture(&w))
                            .unwrap(),
                    )
                    .unwrap();
                }
            }
            if let Some(observer) = &mut diagnostics {
                observer.observe(&w, &events);
            }
            for trace in &w.last_decisions {
                if trace.species != curious::entity::species::Species::Carnivore {
                    continue;
                }
                *predator_reasons
                    .entry(format!("{:?}", trace.decision.reason))
                    .or_insert(0) += 1;
                let ready = trace.before.can_reproduce();
                let mates = trace
                    .seen
                    .entities
                    .iter()
                    .filter(|e| e.species == trace.species && e.reproductive)
                    .count();
                predator_ready += usize::from(ready);
                predator_ready_with_mate += usize::from(ready && mates > 0);
                let history = recent.entry(trace.id).or_default();
                history.push_back(serde_json::json!({"tick": trace.tick, "pos": trace.pos, "life": trace.before,
                    "reason": trace.decision.reason, "ready": ready, "ready_mates": mates,
                    "visible_prey": trace.seen.entities.iter().filter(|e| trace.species.hunts(&e.species)).count()}));
                if history.len() > 10 {
                    history.pop_front();
                }
            }
            for e in events {
                match e {
                    SimulationEvent::EntityDied {
                        id, species, cause, ..
                    } => {
                        *death_causes
                            .entry(format!("{:?}", species))
                            .or_default()
                            .entry(
                                cause
                                    .map(|c| format!("{:?}", c))
                                    .unwrap_or_else(|| "Unknown".into()),
                            )
                            .or_insert(0) += 1;
                        if species == curious::entity::species::Species::Carnivore {
                            let actor = w.entities.iter().find(|e| e.id == id).unwrap();
                            last_predator_deaths.push_back(serde_json::json!({"id": id, "tick": w.tick_counter,
                                "cause": cause, "generation": actor.generation, "parents": actor.parents,
                                "life": actor.base.life(), "last_decisions": recent.remove(&id).unwrap_or_default()}));
                            if last_predator_deaths.len() > 10 {
                                last_predator_deaths.pop_front();
                            }
                        }
                    }
                    SimulationEvent::EntityBorn { species, .. } => {
                        if w.tick_counter > ticks * 9 / 10 {
                            *late_births.entry(format!("{:?}", species)).or_insert(0) += 1;
                        }
                        *births.entry(format!("{:?}", species)).or_insert(0) += 1;
                    }
                    SimulationEvent::EntityMated {
                        parent1, parent2, ..
                    } => {
                        for id in [parent1, parent2] {
                            let p = w.entities.iter().find(|e| e.id == id).unwrap();
                            if p.generation > 0 {
                                breeders.insert(id, format!("{:?}", p.base.species()));
                                if w.tick_counter > ticks * 9 / 10 {
                                    late_breeders.insert(id, format!("{:?}", p.base.species()));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            let living_predators: std::collections::HashSet<_> = w
                .entities
                .iter()
                .filter(|e| {
                    e.base.life().health > 0
                        && e.base.species() == curious::entity::species::Species::Carnivore
                })
                .map(|e| e.id)
                .collect();
            recent.retain(|id, _| living_predators.contains(id));
            let alive_species: std::collections::HashSet<_> = w
                .entities
                .iter()
                .filter(|e| e.base.life().health > 0)
                .map(|e| e.base.species())
                .collect();
            for e in &w.entities {
                let name = format!("{:?}", e.base.species());
                generations
                    .entry(name.clone())
                    .and_modify(|g: &mut usize| *g = (*g).max(e.generation))
                    .or_insert(e.generation);
                if e.base.life().health > 0 {
                    last_alive.insert(name, w.tick_counter);
                }
            }
            if w.tick_counter % sample_every == 0 || w.tick_counter == 1 {
                let mut counts = BTreeMap::<String, usize>::new();
                let mut reserve = BTreeMap::<String, usize>::new();
                let mut sheltered = 0;
                let mut predator_age = [0usize; 3]; // juvenile, adult, final quarter of lifespan
                let mut predator_ready_now = 0;
                let mut predator_energy_blocked = 0;
                for e in &w.entities {
                    if e.base.life().health == 0 {
                        continue;
                    }
                    if e.base.species() == curious::entity::species::Species::Carnivore {
                        let life = e.base.life();
                        let band = if !life.is_mature() {
                            0
                        } else if life.age >= life.max_age * 3 / 4 {
                            2
                        } else {
                            1
                        };
                        predator_age[band] += 1;
                        predator_ready_now += usize::from(life.can_reproduce());
                        predator_energy_blocked += usize::from(
                            life.reproduction_blocker()
                                == Some(
                                    curious::simulation::event::RejectionReason::InsufficientEnergy,
                                ),
                        );
                    }
                    let name = format!("{:?}", e.base.species());
                    *counts.entry(name.clone()).or_insert(0) += 1;
                    *reserve.entry(name).or_insert(0) += e.base.life().energy;
                    if matches!(w.map.cell(e.pos), Some(curious::map::cell::Cell::Burrow)) {
                        sheltered += 1;
                    }
                }
                samples.push(serde_json::json!({"tick":w.tick_counter,"alive":counts,"total_energy":reserve,"sheltered":sheltered,"births":births,"death_causes":death_causes,"predator_age_bands":predator_age,"predator_ready":predator_ready_now,"predator_energy_blocked":predator_energy_blocked}));
            }
            if progress_every > 0 && w.tick_counter % progress_every == 0 {
                let counts = w.entities.iter().filter(|e| e.base.life().health > 0).fold(
                    BTreeMap::<String, usize>::new(),
                    |mut m, e| {
                        *m.entry(format!("{:?}", e.base.species())).or_insert(0) += 1;
                        m
                    },
                );
                eprintln!(
                    "{}",
                    serde_json::json!({"seed": seed, "tick":w.tick_counter,"alive":counts})
                );
            }
            if !alive_species.contains(&curious::entity::species::Species::Carnivore)
                || !alive_species.contains(&curious::entity::species::Species::Herbivore)
            {
                first_extinction.get_or_insert(w.tick_counter);
                if stop_early {
                    break;
                }
            }
        }
        let mut alive: BTreeMap<String, usize> = BTreeMap::new();
        for e in &w.entities {
            if e.base.life().health > 0 {
                *alive.entry(format!("{:?}", e.base.species())).or_insert(0) += 1;
            }
        }
        let mut reproduced: BTreeMap<String, usize> = BTreeMap::new();
        for species in breeders.values() {
            *reproduced.entry(species.clone()).or_insert(0) += 1;
        }
        let mut late_reproduced: BTreeMap<String, usize> = BTreeMap::new();
        for species in late_breeders.values() {
            *late_reproduced.entry(species.clone()).or_insert(0) += 1;
        }
        println!(
            "{}",
            serde_json::json!({"scenario":file,"seed":seed,"mode":mode,"companions":companions,"reserve_opportunities":w.policy.reserve_opportunities,"mate_rendezvous":w.policy.mate_rendezvous,"ticks":w.tick_counter,"requested_ticks":ticks,"first_extinction":first_extinction,"alive":alive,"last_alive":last_alive,"births":births,"descendant_breeders":reproduced,"late_births":late_births,"late_descendant_breeders":late_reproduced,"max_generation":generations,"samples":samples,"death_causes":death_causes,"predator_reasons":predator_reasons,"predator_ready_ticks":predator_ready,"predator_ready_with_mate_ticks":predator_ready_with_mate,"last_predator_deaths":last_predator_deaths,"shock": {"at":shock_at,"duration":shock_duration,"percent":shock_percent},"seconds":start.elapsed().as_secs_f64()})
        );
    }
}
