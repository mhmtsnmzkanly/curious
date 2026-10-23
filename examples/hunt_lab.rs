//! Fixed bounded physical hunt laboratory. See the versioned protocol files.
use curious::{
    Result,
    creatures::{carnivore::CarnivoreEntity, herbivore::HerbivoreEntity},
    entity::{
        controller::EntityController,
        decision::{Decision, Target},
        hunt_learning::{HuntEnd, HuntLearner, HuntResult, HuntRiskValue},
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
const V3_PAIRS: [(u64, u64); 8] = [
    (1, 43001),
    (2, 43002),
    (3, 43003),
    (4, 43004),
    (5, 43005),
    (42, 43006),
    (12345, 43007),
    (2026, 43008),
];
const ARMS: [&str; 4] = [
    "rule_based",
    "naive_online",
    "experienced_online",
    "experienced_frozen",
];
const V3_ARMS: [&str; 6] = [
    "rule_based",
    "naive_online",
    "experienced_online",
    "experienced_risk_online",
    "experienced_risk_frozen",
    "experienced_risk_ablation_frozen",
];
fn visits(l: &HuntLearner) -> u64 {
    l.values.iter().map(|v| u64::from(v.visits)).sum()
}
fn risk_totals(values: &[HuntRiskValue]) -> (u64, u64, u64) {
    values
        .iter()
        .fold((0, 0, 0), |(exposures, injuries, damage), value| {
            (
                exposures + u64::from(value.exposures),
                injuries + u64::from(value.injuries),
                damage + value.damage,
            )
        })
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
    prey_defense_damage: usize,
    external_damage: usize,
    unattributed_damage: usize,
    rejected: usize,
    energy: usize,
    stamina: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    risk_explored_unknown: Option<bool>,
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
    prey_defense_damage: usize,
    external_damage: usize,
    unattributed_damage: usize,
    rejected: usize,
    hunter_died: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    final_health: Option<usize>,
    first_food_tick: Option<usize>,
    failed_ticks: usize,
    stop: String,
    visits_before: u64,
    visits_after: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    risk_exposures_before: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    risk_exposures_after: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    risk_injuries_before: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    risk_injuries_after: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    risk_damage_before: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    risk_damage_after: Option<u64>,
    endings: Vec<EndAudit>,
    timeline: Vec<TickAudit>,
}
fn audited_hunt_target(current: Option<usize>, events: &[SimulationEvent]) -> Option<usize> {
    current.or_else(|| {
        events.iter().rev().find_map(|event| match event {
            SimulationEvent::HuntEpisodeEnded { id: 1, result }
                if result.end != HuntEnd::Interrupted =>
            {
                Some(result.attempt.target)
            }
            _ => None,
        })
    })
}
fn trial(
    learner: &mut HuntLearner,
    seed: u64,
    number: usize,
    reversed: bool,
    defensive: bool,
    sample: Option<&Path>,
    audit_v3: bool,
) -> Result<Trial> {
    learner.episode = None;
    learner.last_episode = None;
    learner.take_results();
    let before = visits(learner);
    let risk_before = risk_totals(&learner.risk_values);
    let audit_risk = learner.risk_enabled || !learner.risk_values.is_empty();
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
        prey_defense_damage: 0,
        external_damage: 0,
        unattributed_damage: 0,
        rejected: 0,
        energy_spent: 0,
        non_action_energy: 0,
        hunter_died: false,
        final_health: None,
        first_food_tick: None,
        failed_ticks: 0,
        stop: "TrialLimit".into(),
        visits_before: before,
        visits_after: before,
        risk_exposures_before: audit_risk.then_some(risk_before.0),
        risk_exposures_after: audit_risk.then_some(risk_before.0),
        risk_injuries_before: audit_risk.then_some(risk_before.1),
        risk_injuries_after: audit_risk.then_some(risk_before.1),
        risk_damage_before: audit_risk.then_some(risk_before.2),
        risk_damage_after: audit_risk.then_some(risk_before.2),
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
            episode_target: audited_hunt_target(
                actor.hunt_learner.episode.as_ref().map(|e| e.target),
                &events,
            ),
            movement_energy: 0,
            attack_energy: 0,
            food: 0,
            incoming_damage: 0,
            prey_defense_damage: 0,
            external_damage: 0,
            unattributed_damage: 0,
            rejected: 0,
            energy: actor.base.life().energy,
            stamina: actor.base.life().stamina,
            risk_explored_unknown: actor
                .hunt_learner
                .risk_enabled
                .then_some(actor.hunt_learner.last_choice_explored_unknown),
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
                    attacker,
                    target: 1,
                    damage,
                    ..
                } => {
                    tick.incoming_damage += damage;
                    match tick.episode_target {
                        Some(target) if target == *attacker => tick.prey_defense_damage += damage,
                        Some(_) => tick.external_damage += damage,
                        None => tick.unattributed_damage += damage,
                    }
                }
                SimulationEvent::EntityAte { id: 1, amount, .. } => tick.food += amount,
                SimulationEvent::ActionRejected { id: 1, .. } => tick.rejected += 1,
                SimulationEvent::HuntEpisodeEnded { id: 1, result } => {
                    let reason = trace.map(|t| format!("{:?}", t.decision.reason));
                    let trigger = match reason.as_deref() {
                        Some("HuntLowEnergy" | "HuntTooCostly")
                            if result.end == HuntEnd::Interrupted =>
                        {
                            format!("OwnBudgetCutoff:{}", reason.unwrap())
                        }
                        Some("LearnedHunt") if result.end == HuntEnd::Interrupted => {
                            "TargetChanged".into()
                        }
                        Some(other) => other.to_string(),
                        None => "BeforeDecisionDeath".into(),
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
        r.prey_defense_damage += tick.prey_defense_damage;
        r.external_damage += tick.external_damage;
        r.unattributed_damage += tick.unattributed_damage;
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
        actor.hunt_learner.interrupt_censored(HuntEnd::Interrupted);
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
    r.final_health = audit_v3.then_some(actor.base.life().health);
    let risk_after = risk_totals(&actor.hunt_learner.risk_values);
    if audit_risk || actor.hunt_learner.risk_enabled || !actor.hunt_learner.risk_values.is_empty() {
        r.risk_exposures_before = Some(risk_before.0);
        r.risk_exposures_after = Some(risk_after.0);
        r.risk_injuries_before = Some(risk_before.1);
        r.risk_injuries_after = Some(risk_after.1);
        r.risk_damage_before = Some(risk_before.2);
        r.risk_damage_after = Some(risk_after.2);
    }
    r.failed_ticks = if r.food == 0 { r.ticks } else { 0 };
    *learner = std::mem::take(&mut actor.hunt_learner);
    Ok(r)
}
fn run_pair(
    training_seed: u64,
    evaluation_seed: u64,
    track: &str,
    samples: Option<&Path>,
    protocol_v2: bool,
) -> Result<serde_json::Value> {
    let defensive = track == "defensive";
    let mut trained = HuntLearner {
        enabled: true,
        ..Default::default()
    };
    let training: Vec<_> = schedule(training_seed, TRAIN)
        .into_iter()
        .enumerate()
        .map(|(i, seed)| trial(&mut trained, seed, i, false, defensive, None, false))
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
            let sample = if training_seed == PAIRS[0].0 && [0, REVERSAL].contains(&i) {
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
                false,
            )?);
        }
        arms.push(serde_json::json!({"arm":name,"initial_values":initial_values,"final_values":learner.values,"trials":trials}));
    }
    Ok(
        serde_json::json!({"schema":if protocol_v2 {"curious-hunt-lab-v2"} else {"curious-hunt-lab-v1"},"track":track,"training_seed":training_seed,"evaluation_seed":evaluation_seed,"training_trials":TRAIN,"evaluation_trials":EVAL,"reversal_at":REVERSAL,"max_trial_ticks":LIMIT,"controlled_body_reset":true,"training":training,"training_values":training_values,"arms":arms}),
    )
}
fn run_pair_v3(training_seed: u64, evaluation_seed: u64, track: &str) -> Result<serde_json::Value> {
    let defensive = track == "defensive";
    let seeds = schedule(training_seed, TRAIN);
    let mut plain = HuntLearner {
        enabled: true,
        ..Default::default()
    };
    let mut risk = HuntLearner {
        enabled: true,
        risk_enabled: true,
        ..Default::default()
    };
    let mut plain_training = Vec::with_capacity(TRAIN);
    let mut risk_training = Vec::with_capacity(TRAIN);
    for (i, seed) in seeds.into_iter().enumerate() {
        plain_training.push(trial(&mut plain, seed, i, false, defensive, None, true)?);
        risk_training.push(trial(&mut risk, seed, i, false, defensive, None, true)?);
    }
    let plain_values = plain.values.clone();
    let risk_values = risk.values.clone();
    let risk_table = risk.risk_values.clone();
    let eval_seeds = schedule(evaluation_seed, EVAL);
    let mut arms = Vec::with_capacity(V3_ARMS.len());
    for name in V3_ARMS {
        let mut learner = match name {
            "rule_based" => HuntLearner::default(),
            "naive_online" => HuntLearner {
                enabled: true,
                ..Default::default()
            },
            "experienced_online" => plain.clone(),
            "experienced_risk_online" => risk.clone(),
            "experienced_risk_frozen" | "experienced_risk_ablation_frozen" => risk.clone(),
            _ => unreachable!(),
        };
        learner.update_enabled = !matches!(
            name,
            "experienced_risk_frozen" | "experienced_risk_ablation_frozen"
        );
        if name == "experienced_risk_ablation_frozen" {
            learner.risk_enabled = false;
        }
        let initial_values = learner.values.clone();
        let initial_risk_values = learner.risk_values.clone();
        let mut trials = Vec::with_capacity(EVAL);
        for (i, seed) in eval_seeds.iter().enumerate() {
            trials.push(trial(
                &mut learner,
                *seed,
                i,
                i >= REVERSAL,
                defensive,
                None,
                true,
            )?);
        }
        arms.push(serde_json::json!({
            "arm":name,
            "enabled":learner.enabled,
            "risk_enabled":learner.risk_enabled,
            "update_enabled":learner.update_enabled,
            "initial_values":initial_values,
            "final_values":learner.values,
            "initial_risk_values":initial_risk_values,
            "final_risk_values":learner.risk_values,
            "trials":trials
        }));
    }
    Ok(serde_json::json!({
        "schema":"curious-hunt-lab-v3",
        "track":track,
        "training_seed":training_seed,
        "evaluation_seed":evaluation_seed,
        "training_trials":TRAIN,
        "evaluation_trials":EVAL,
        "reversal_at":REVERSAL,
        "max_trial_ticks":LIMIT,
        "controlled_body_reset":true,
        "training":plain_training,
        "risk_training":risk_training,
        "training_values":plain_values,
        "risk_training_values":risk_values,
        "risk_training_risk_values":risk_table,
        "arms":arms
    }))
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let mut samples = None;
    let mut evaluation_seed_start = None;
    let mut protocol_version = 1u8;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--samples" if i + 1 < args.len() && samples.is_none() => {
                samples = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            "--evaluation-seed-start" if i + 1 < args.len() && evaluation_seed_start.is_none() => {
                evaluation_seed_start = Some(args[i + 1].parse::<u64>()?);
                i += 2;
            }
            "--protocol-version" if i + 1 < args.len() && protocol_version == 1 => {
                protocol_version = args[i + 1].parse::<u8>()?;
                if !matches!(protocol_version, 1 | 2 | 3) { return Err("protocol version must be 1, 2, or 3".into()); }
                i += 2;
            }
            _ => {
                return Err(
                    "usage: hunt_lab [--protocol-version 1|2|3] [--samples NEW_DIRECTORY] [--evaluation-seed-start SEED]"
                        .into(),
                )
            }
        }
    }
    if protocol_version < 3 && evaluation_seed_start.is_some() {
        protocol_version = 2;
    }
    if protocol_version == 3 && evaluation_seed_start.is_none() {
        return Err("protocol v3 requires --evaluation-seed-start".into());
    }
    let source_pairs = if protocol_version == 3 {
        &V3_PAIRS[..]
    } else {
        &PAIRS[..]
    };
    let pairs = evaluation_seed_start.map_or(source_pairs.to_vec(), |start| {
        source_pairs
            .iter()
            .enumerate()
            .map(|(i, (training, _))| (*training, start + i as u64))
            .collect()
    });
    if let Some(root) = &samples {
        fs::create_dir(root)?;
    }
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    for track in ["passive", "defensive"] {
        for (a, b) in &pairs {
            let result = if protocol_version == 3 {
                run_pair_v3(*a, *b, track)?
            } else {
                run_pair(*a, *b, track, samples.as_deref(), protocol_version == 2)?
            };
            serde_json::to_writer(&mut out, &result)?;
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
mod attribution_tests {
    use super::*;
    use curious::entity::hunt_learning::{HuntContext, HuntEpisode, HuntResult};

    fn ending(end: HuntEnd, target: usize) -> SimulationEvent {
        SimulationEvent::HuntEpisodeEnded {
            id: 1,
            result: HuntResult {
                attempt: HuntEpisode {
                    target,
                    context: HuntContext {
                        energy: 1,
                        stamina: 1,
                        distance: 1,
                        health: 1,
                    },
                    ticks: 1,
                    initial_energy: 10,
                    initial_stamina: 10,
                    movement_energy: 1,
                    attack_energy: 0,
                    incoming_damage: 0,
                    prey_defense_damage: 0,
                    food: 0,
                    own_kill: false,
                    ambiguous: false,
                },
                end,
                reward: None,
                updated: false,
            },
        }
    }

    #[test]
    fn predecision_interrupt_does_not_claim_resolver_damage_target() {
        let events = [ending(HuntEnd::Interrupted, 10)];
        assert_eq!(audited_hunt_target(None, &events), None);
        assert_eq!(audited_hunt_target(Some(20), &events), Some(20));
    }

    #[test]
    fn fed_or_timed_out_closure_keeps_target_for_last_tick_attribution() {
        let fed = [ending(HuntEnd::Fed, 10)];
        let timed = [ending(HuntEnd::TimedOut, 20)];
        assert_eq!(audited_hunt_target(None, &fed), Some(10));
        assert_eq!(audited_hunt_target(None, &timed), Some(20));
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
        trial(&mut trained, 1, 0, true, false, None, false).unwrap();
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
        let trained = trial(&mut learner, 50, 0, true, false, None, false).unwrap();
        assert!(trained.food > 0);
        assert_eq!(
            trained.endings.iter().filter(|e| e.result.updated).count() as u64,
            trained.visits_after - trained.visits_before
        );
        learner.update_enabled = false;
        let values = learner.values.clone();
        let r = trial(&mut learner, 50, 0, true, false, None, false).unwrap();
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
        let r = trial(&mut learner, 7, 0, false, true, None, false).unwrap();
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
