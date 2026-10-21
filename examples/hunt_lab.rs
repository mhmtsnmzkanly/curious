//! Fixed four-arm physical hunt laboratory. See docs/HUNT_LAB_PROTOCOL.md.
use curious::{
    Result,
    creatures::{carnivore::CarnivoreEntity, herbivore::HerbivoreEntity},
    entity::{
        controller::EntityController,
        decision::{Decision, Target},
        hunt_learning::{HuntEnd, HuntLearner, HuntResult},
        intent::Intent,
        phase::EntityPhase,
    },
    logger::Logger,
    map::{
        Map,
        movement::{Direction, Position, Steps},
    },
    persistence::Store,
    random::Random,
    simulation::SimulationEvent,
    systems::perception::PerceptionSystem,
    world::{EntitySlot, World},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
const TRAIN: usize = 80;
const EVAL: usize = 80;
const REVERSAL: usize = 40;
const LIMIT: usize = 20;
const PAIRS: [(u64, u64); 8] = [
    (1, 41001),
    (2, 41002),
    (3, 41003),
    (4, 41004),
    (5, 41005),
    (42, 41006),
    (12345, 41007),
    (2026, 41008),
];
const ARMS: [&str; 4] = [
    "rule_based",
    "naive_online",
    "experienced_online",
    "experienced_frozen",
];
fn visits(l: &HuntLearner) -> u64 {
    l.values.iter().map(|v| u64::from(v.visits)).sum()
}
fn schedule(seed: u64, n: usize) -> Vec<u64> {
    let mut rng = Random::new(seed);
    (0..n).map(|_| rng.next_u64()).collect()
}
fn world(seed: u64, learner: HuntLearner) -> World {
    let mut hunter = CarnivoreEntity::default();
    hunter.life_state.energy = 40;
    hunter.life_state.speed = 1;
    let mut actor = EntitySlot::new(
        1,
        Position::new(2, 2),
        EntityPhase::Active,
        EntityController::AI,
        Box::new(hunter),
    );
    actor.hunt_learner = learner;
    let mut actors = vec![actor];
    for (id, pos, health) in [(2, Position::new(3, 2), 20), (3, Position::new(2, 5), 60)] {
        let mut prey = HerbivoreEntity::default();
        prey.life_state.health = health;
        prey.life_state.max_energy = 300;
        prey.life_state.energy = 60;
        prey.life_state.max_stamina = 64;
        prey.life_state.stamina = 64;
        prey.life_state.speed = 2;
        actors.push(EntitySlot::new(
            id,
            pos,
            EntityPhase::Active,
            EntityController::Player(id as u32),
            Box::new(prey),
        ));
    }
    let mut w = World::with_seed(0, 12, 0, 12, actors, seed);
    w.logger = Logger::disabled();
    w.map = Map::new(0, 12, 0, 12);
    w.environment = Default::default();
    w.genetics.enabled = false;
    for s in &mut w.entities {
        s.learner.enabled = false;
    }
    w
}
fn commands(w: &mut World, step: usize, reversed: bool, defensive: bool) {
    let hunter = w.entities.iter().find(|s| s.id == 1).unwrap().pos;
    for id in [2, 3] {
        let Some(s) = w
            .entities
            .iter()
            .find(|s| s.id == id && s.phase.is_active())
        else {
            continue;
        };
        let fleeing = if reversed { id == 3 } else { id == 2 };
        let intent = if fleeing {
            let direction = match (step % 8) / 2 {
                0 => Direction::Right,
                1 => Direction::Down,
                2 => Direction::Left,
                _ => Direction::Up,
            };
            Intent::Move {
                steps: Steps(vec![direction; 2]),
            }
        } else if defensive && (s.pos.x - hunter.x).abs() <= 1 && (s.pos.y - hunter.y).abs() <= 1 {
            Intent::Attack { target_id: 1 }
        } else {
            Intent::Idle { duration: 1 }
        };
        w.push_command(id, intent);
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct EndAudit {
    origin: String,
    trigger: String,
    result: HuntResult,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct TickAudit {
    tick: usize,
    decision: Option<Decision>,
    target: Option<Target>,
    episode_target: Option<usize>,
    movement_energy: usize,
    attack_energy: usize,
    food: usize,
    incoming_damage: usize,
    rejected: usize,
    energy: usize,
    stamina: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Trial {
    trial: usize,
    world_seed: u64,
    reversed: bool,
    initial_bodies: serde_json::Value,
    initial_seen: serde_json::Value,
    ticks: usize,
    food: usize,
    movement_energy: usize,
    attack_energy: usize,
    energy_spent: usize,
    non_action_energy: usize,
    incoming_damage: usize,
    rejected: usize,
    hunter_died: bool,
    first_food_tick: Option<usize>,
    failed_ticks: usize,
    stop: String,
    visits_before: u64,
    visits_after: u64,
    endings: Vec<EndAudit>,
    timeline: Vec<TickAudit>,
}
fn trial(
    learner: &mut HuntLearner,
    seed: u64,
    number: usize,
    reversed: bool,
    defensive: bool,
    sample: Option<&Path>,
) -> Result<Trial> {
    learner.episode = None;
    learner.last_episode = None;
    learner.take_results();
    let before = visits(learner);
    let mut w = world(seed, std::mem::take(learner));
    let initial_energy = w.entities[0].base.life().energy;
    let initial_bodies=serde_json::json!(w.entities.iter().map(|s|serde_json::json!({"id":s.id,"life":s.base.life(),"pos":s.pos,"phase":s.phase,"species":s.base.species()})).collect::<Vec<_>>());
    let initial_seen = serde_json::to_value(PerceptionSystem::build(&w, &w.entities[0]).entities)?;
    let mut store = if let Some(path) = sample {
        if path.exists() {
            return Err("sample directory must be fresh".into());
        }
        let mut db = Store::open(path, 16 * 1024 * 1024)?;
        db.record(&w, &[], "lab_initial", 0, None)?;
        Some(db)
    } else {
        None
    };
    let mut r = Trial {
        trial: number,
        world_seed: seed,
        reversed,
        initial_bodies,
        initial_seen,
        ticks: 0,
        food: 0,
        movement_energy: 0,
        attack_energy: 0,
        incoming_damage: 0,
        rejected: 0,
        energy_spent: 0,
        non_action_energy: 0,
        hunter_died: false,
        first_food_tick: None,
        failed_ticks: 0,
        stop: "TrialLimit".into(),
        visits_before: before,
        visits_after: before,
        endings: vec![],
        timeline: vec![],
    };
    for step in 0..LIMIT {
        commands(&mut w, step, reversed, defensive);
        let events = w.tick();
        r.ticks += 1;
        let trace = w.last_decisions.iter().find(|t| t.id == 1);
        let actor = w.entities.iter().find(|s| s.id == 1).unwrap();
        let mut tick = TickAudit {
            tick: w.tick_counter,
            decision: trace.map(|t| t.decision.clone()),
            target: trace.and_then(|t| t.target.clone()),
            episode_target: actor.hunt_learner.episode.as_ref().map(|e| e.target),
            movement_energy: 0,
            attack_energy: 0,
            food: 0,
            incoming_damage: 0,
            rejected: 0,
            energy: actor.base.life().energy,
            stamina: actor.base.life().stamina,
        };
        for event in &events {
            match event {
                SimulationEvent::EntityMoved {
                    id: 1, energy_cost, ..
                } => tick.movement_energy += energy_cost,
                SimulationEvent::EntityAttacked {
                    attacker: 1,
                    energy_cost,
                    ..
                } => tick.attack_energy += energy_cost,
                SimulationEvent::EntityAttacked {
                    target: 1, damage, ..
                } => tick.incoming_damage += damage,
                SimulationEvent::EntityAte { id: 1, amount, .. } => tick.food += amount,
                SimulationEvent::ActionRejected { id: 1, .. } => tick.rejected += 1,
                SimulationEvent::HuntEpisodeEnded { id: 1, result } => {
                    let trigger = if result.end == HuntEnd::Interrupted
                        && trace
                            .is_some_and(|t| format!("{:?}", t.decision.reason) == "LearnedHunt")
                    {
                        "TargetChanged".into()
                    } else {
                        trace
                            .map(|t| format!("{:?}", t.decision.reason))
                            .unwrap_or_else(|| "BeforeDecisionDeath".into())
                    };
                    r.endings.push(EndAudit {
                        origin: "engine".into(),
                        trigger,
                        result: result.clone(),
                    });
                }
                _ => {}
            }
        }
        r.food += tick.food;
        r.movement_energy += tick.movement_energy;
        r.attack_energy += tick.attack_energy;
        r.incoming_damage += tick.incoming_damage;
        r.rejected += tick.rejected;
        if tick.food > 0 && r.first_food_tick.is_none() {
            r.first_food_tick = Some(w.tick_counter);
        }
        r.hunter_died = !actor.base.life().is_alive();
        r.timeline.push(tick);
        if let Some(s) = &mut store {
            s.record(&w, &events, "tick", 0, None)?;
        }
        if r.food > 0 {
            r.stop = "FirstFood".into();
            break;
        }
        if r.hunter_died {
            r.stop = "HunterDied".into();
            break;
        }
    }
    let actor = w.entities.iter_mut().find(|s| s.id == 1).unwrap();
    if actor.hunt_learner.episode.is_some() {
        actor.hunt_learner.interrupt(HuntEnd::Interrupted);
        let ended = actor.hunt_learner.take_results();
        let mut events = vec![];
        for result in ended {
            r.endings.push(EndAudit {
                origin: "lab_boundary".into(),
                trigger: r.stop.clone(),
                result: result.clone(),
            });
            events.push(SimulationEvent::HuntEpisodeEnded { id: 1, result });
        }
        if let Some(s) = &mut store {
            s.record(&w, &events, "lab_boundary", 0, None)?;
        }
    }
    let actor = w.entities.iter_mut().find(|s| s.id == 1).unwrap();
    actor.hunt_learner.validate()?;
    r.energy_spent = initial_energy + r.food - actor.base.life().energy;
    r.non_action_energy = r
        .energy_spent
        .checked_sub(r.movement_energy + r.attack_energy)
        .ok_or("action energy exceeds body accounting")?;
    r.visits_after = visits(&actor.hunt_learner);
    r.failed_ticks = if r.food == 0 { r.ticks } else { 0 };
    *learner = std::mem::take(&mut actor.hunt_learner);
    Ok(r)
}
fn run_pair(
    training_seed: u64,
    evaluation_seed: u64,
    track: &str,
    samples: Option<&Path>,
) -> Result<serde_json::Value> {
    let defensive = track == "defensive";
    let mut trained = HuntLearner {
        enabled: true,
        ..Default::default()
    };
    let training: Vec<_> = schedule(training_seed, TRAIN)
        .into_iter()
        .enumerate()
        .map(|(i, seed)| trial(&mut trained, seed, i, false, defensive, None))
        .collect::<Result<_>>()?;
    let training_values = trained.values.clone();
    let eval_seeds = schedule(evaluation_seed, EVAL);
    let mut arms = vec![];
    for name in ARMS {
        let mut learner = match name {
            "rule_based" => HuntLearner::default(),
            "naive_online" => HuntLearner {
                enabled: true,
                ..Default::default()
            },
            _ => trained.clone(),
        };
        learner.update_enabled = name != "experienced_frozen";
        let initial_values = learner.values.clone();
        let mut trials = vec![];
        for (i, seed) in eval_seeds.iter().enumerate() {
            let sample = if evaluation_seed == PAIRS[0].1 && [0, REVERSAL].contains(&i) {
                samples.map(|root| root.join(track).join(name).join(format!("trial-{i:03}")))
            } else {
                None
            };
            if let Some(p) = &sample {
                fs::create_dir_all(p.parent().unwrap())?;
            }
            trials.push(trial(
                &mut learner,
                *seed,
                i,
                i >= REVERSAL,
                defensive,
                sample.as_deref(),
            )?);
        }
        arms.push(serde_json::json!({"arm":name,"initial_values":initial_values,"final_values":learner.values,"trials":trials}));
    }
    Ok(
        serde_json::json!({"schema":"curious-hunt-lab-v1","track":track,"training_seed":training_seed,"evaluation_seed":evaluation_seed,"training_trials":TRAIN,"evaluation_trials":EVAL,"reversal_at":REVERSAL,"max_trial_ticks":LIMIT,"controlled_body_reset":true,"training":training,"training_values":training_values,"arms":arms}),
    )
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let samples = match args.as_slice() {
        [] => None,
        [flag, path] if flag == "--samples" => Some(PathBuf::from(path)),
        _ => return Err("usage: hunt_lab [--samples NEW_DIRECTORY]".into()),
    };
    if let Some(root) = &samples {
        fs::create_dir(root)?;
    }
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    for track in ["passive", "defensive"] {
        for (a, b) in PAIRS {
            serde_json::to_writer(&mut out, &run_pair(a, b, track, samples.as_deref())?)?;
            writeln!(out)?;
            out.flush()?;
        }
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use curious::simulation::checkpoint::State;
    #[test]
    fn same_body_and_complete_perception_across_histories() {
        let mut trained = HuntLearner {
            enabled: true,
            ..Default::default()
        };
        trial(&mut trained, 1, 0, true, false, None).unwrap();
        let mut a = world(42, HuntLearner::default());
        let mut b = world(42, trained);
        assert_eq!(
            serde_json::to_value(a.entities[0].base.life()).unwrap(),
            serde_json::to_value(b.entities[0].base.life()).unwrap()
        );
        assert_eq!(
            serde_json::to_value(PerceptionSystem::build(&a, &a.entities[0])).unwrap(),
            serde_json::to_value(PerceptionSystem::build(&b, &b.entities[0])).unwrap()
        );
        a.tick();
        b.tick();
        assert_eq!(
            a.last_decisions[0].before.age,
            b.last_decisions[0].before.age
        );
    }
    #[test]
    fn real_food_audit_matches_updates_and_frozen_is_unchanged() {
        let mut learner = HuntLearner {
            enabled: true,
            ..Default::default()
        };
        let trained = trial(&mut learner, 50, 0, true, false, None).unwrap();
        assert!(trained.food > 0);
        assert_eq!(
            trained.endings.iter().filter(|e| e.result.updated).count() as u64,
            trained.visits_after - trained.visits_before
        );
        learner.update_enabled = false;
        let values = learner.values.clone();
        let r = trial(&mut learner, 50, 0, true, false, None).unwrap();
        assert_eq!(learner.values, values);
        assert_eq!(r.visits_before, r.visits_after);
        assert!(r.endings.iter().all(|e| !e.result.updated));
    }
    #[test]
    fn scripted_escape_obeys_the_resolver_and_pays_real_energy() {
        let mut w = world(
            50,
            HuntLearner {
                enabled: true,
                ..Default::default()
            },
        );
        commands(&mut w, 0, false, false);
        let events = w.tick();
        assert!(events.iter().any(|e| matches!(
            e,
            SimulationEvent::EntityMoved {
                id: 2,
                energy_cost: 2,
                ..
            }
        )));
        let prey = w.entities.iter().find(|s| s.id == 2).unwrap();
        assert_eq!(prey.pos, Position::new(5, 2));
        assert_eq!(prey.base.life().energy, 57);
    }
    #[test]
    fn scripted_moves_are_real_costs_and_clock_is_bounded() {
        let mut learner = HuntLearner {
            enabled: true,
            ..Default::default()
        };
        let r = trial(&mut learner, 7, 0, false, true, None).unwrap();
        assert!(r.ticks <= LIMIT);
        assert_eq!(r.ticks, r.timeline.len());
        assert_eq!(
            r.movement_energy,
            r.timeline.iter().map(|t| t.movement_energy).sum::<usize>()
        );
        assert!(
            r.endings
                .iter()
                .filter(|e| e.origin == "lab_boundary")
                .all(|e| e.result.reward.is_none() && !e.result.updated)
        );
    }
    #[test]
    fn audit_queue_drains_and_world_checkpoint_replays_events() {
        let mut w = world(
            50,
            HuntLearner {
                enabled: true,
                ..Default::default()
            },
        );
        commands(&mut w, 0, true, false);
        w.tick();
        assert!(w.entities[0].hunt_learner.pending_results.is_empty());
        let mut restored = State::capture(&w).restore().unwrap();
        for t in 1..5 {
            commands(&mut w, t, true, false);
            commands(&mut restored, t, true, false);
            let a = w.tick();
            let b = restored.tick();
            assert_eq!(
                serde_json::to_value(a).unwrap(),
                serde_json::to_value(b).unwrap()
            );
            assert_eq!(
                serde_json::to_value(State::capture(&w)).unwrap(),
                serde_json::to_value(State::capture(&restored)).unwrap()
            );
        }
    }
}
