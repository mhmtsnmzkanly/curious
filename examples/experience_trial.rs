//! Predeclared, bounded water-learning laboratory; bodies/resources reset each episode.
//! Prints JSONL, one held-out seed pair per line. This is not a living ecosystem.
use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::{
        controller::EntityController,
        decision::Target,
        intent::Intent,
        learning::{Learner, WaterEnd},
        phase::EntityPhase,
    },
    map::{
        Map,
        cell::Cell,
        movement::{Direction, Position, Steps},
    },
    simulation::SimulationEvent,
    world::{EntitySlot, World},
};
use serde::{Deserialize, Serialize};

const TRAIN_TRIALS: usize = 400;
const EVAL_TRIALS: usize = 400;
const REVERSAL: usize = 200;
const PAIRS: [(u64, u64); 8] = [
    (1, 30001),
    (2, 30002),
    (3, 30003),
    (4, 30004),
    (5, 30005),
    (42, 30006),
    (12345, 30007),
    (2026, 30008),
];

fn laboratory(seed: u64, learner: Learner) -> World {
    let actors = vec![
        EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        ),
        EntitySlot::new(
            2,
            Position::new(2, 0),
            EntityPhase::Active,
            EntityController::Player(0),
            Box::new(HerbivoreEntity::default()),
        ),
    ];
    let mut world = World::with_seed(0, 4, 0, 4, actors, seed);
    world.logger = curious::logger::Logger::disabled();
    world.environment = Default::default();
    world.entities[0].learner = learner;
    world
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Trial {
    trial: usize,
    reversed: bool,
    target: Option<Position>,
    water: usize,
    movement_energy: usize,
    incoming_damage: usize,
    rejected_actions: usize,
    success: bool,
    partial: bool,
    zero_water: bool,
    /// Explicit laboratory truncation, not scored as a biological failure.
    unfinished_episode: bool,
    end: Option<WaterEnd>,
    reward: Option<i32>,
    visits_before: u64,
    visits_after: u64,
}
fn visits(learner: &Learner) -> u64 {
    learner
        .values
        .iter()
        .map(|value| u64::from(value.visits))
        .sum()
}

fn trial(world: &mut World, number: usize, reversed: bool) -> Trial {
    // Only learned values persist. Equal age, physiology, phase, location and memory.
    world.map = Map::new(0, 4, 0, 4);
    world
        .map
        .set_cell(Position::new(1, 0), Cell::Water { amount: 10 });
    world
        .map
        .set_cell(Position::new(0, 3), Cell::Water { amount: 10 });
    for slot in &mut world.entities {
        slot.base = Box::new(HerbivoreEntity::default());
        slot.base.life_mut().age = 0;
        slot.base.life_mut().water = 10;
        slot.phase = EntityPhase::Active;
        slot.memory = Default::default();
        slot.learner.episode = None;
        slot.learner.last_episode = None;
    }
    world.entities[0].pos = Position::new(0, 0);
    world.entities[1].pos = if reversed {
        Position::new(0, 4)
    } else {
        Position::new(2, 0)
    };
    let before = visits(&world.entities[0].learner);
    world.push_command(
        2,
        Intent::Drink {
            at: Steps(vec![if reversed {
                Direction::Up
            } else {
                Direction::Left
            }]),
        },
    );
    let events = world.tick();
    let target = world
        .last_decisions
        .iter()
        .find(|decision| decision.id == 1)
        .and_then(|decision| match decision.target {
            Some(Target::Position(pos)) => Some(pos),
            _ => None,
        });
    let mut result = Trial {
        trial: number,
        reversed,
        target,
        water: 0,
        movement_energy: 0,
        incoming_damage: 0,
        rejected_actions: 0,
        success: false,
        partial: false,
        zero_water: true,
        unfinished_episode: world.entities[0].learner.episode.is_some(),
        end: world.entities[0]
            .learner
            .last_episode
            .as_ref()
            .map(|outcome| outcome.end),
        reward: world.entities[0]
            .learner
            .last_episode
            .as_ref()
            .and_then(|outcome| outcome.reward),
        visits_before: before,
        visits_after: visits(&world.entities[0].learner),
    };
    for event in events {
        match event {
            SimulationEvent::EntityDrank { id: 1, amount, .. } => result.water += amount,
            SimulationEvent::EntityMoved {
                id: 1, energy_cost, ..
            } => result.movement_energy += energy_cost,
            SimulationEvent::EntityAttacked {
                target: 1, damage, ..
            } => result.incoming_damage += damage,
            SimulationEvent::ActionRejected { id: 1, .. } => result.rejected_actions += 1,
            _ => {}
        }
    }
    result.success = result.water == 10;
    result.partial = result.water > 0 && result.water < 10;
    result.zero_water = result.water == 0;
    if result.unfinished_episode {
        world.entities[0].learner.interrupt(WaterEnd::Interrupted);
    }
    result
}

fn summarize(rows: &[Trial]) -> serde_json::Value {
    let near = rows
        .iter()
        .filter(|row| row.target == Some(Position::new(1, 0)))
        .count();
    let far = rows
        .iter()
        .filter(|row| row.target == Some(Position::new(0, 3)))
        .count();
    serde_json::json!({
        "trials": rows.len(), "water": rows.iter().map(|row| row.water).sum::<usize>(),
        "movement_energy": rows.iter().map(|row| row.movement_energy).sum::<usize>(),
        "incoming_damage": rows.iter().map(|row| row.incoming_damage).sum::<usize>(),
        "rejected_actions": rows.iter().map(|row| row.rejected_actions).sum::<usize>(),
        "full_drinks": rows.iter().filter(|row| row.success).count(),
        "partial_drinks": rows.iter().filter(|row| row.partial).count(),
        "zero_water": rows.iter().filter(|row| row.zero_water).count(),
        "unfinished_episodes": rows.iter().filter(|row| row.unfinished_episode).count(),
        "scored_episodes": rows.iter().filter(|row| row.reward.is_some()).count(),
        "new_experiences": rows.iter().map(|row| row.visits_after - row.visits_before).sum::<u64>(),
        "near_choices": near, "far_choices": far,
        "other_choices": rows.len() - near - far,
    })
}

fn experiment(training_seed: u64, evaluation_seed: u64) -> serde_json::Value {
    let mut training = laboratory(training_seed, Learner::default());
    let training_rows: Vec<_> = (0..TRAIN_TRIALS)
        .map(|i| trial(&mut training, i, false))
        .collect();
    let trained = training.entities[0].learner.clone();
    let mut arms = Vec::new();
    for arm in [
        "rule_based",
        "naive_online",
        "experienced_online",
        "experienced_frozen",
    ] {
        let mut learner = if arm.starts_with("experienced") {
            trained.clone()
        } else {
            Learner::default()
        };
        learner.episode = None;
        learner.last_episode = None;
        learner.enabled = arm != "rule_based";
        learner.update_enabled = arm != "experienced_frozen";
        let initial_values = learner.values.clone();
        let initial_visits = visits(&learner);
        let mut evaluation = laboratory(evaluation_seed, learner);
        let mut rows = Vec::new();
        let mut reversal_values = serde_json::Value::Null;
        for i in 0..EVAL_TRIALS {
            rows.push(trial(&mut evaluation, i, i >= REVERSAL));
            if i + 1 == REVERSAL {
                reversal_values =
                    serde_json::to_value(&evaluation.entities[0].learner.values).unwrap();
            }
        }
        arms.push(serde_json::json!({
            "arm": arm, "initial_values": initial_values, "initial_visits": initial_visits,
            "reversal_values": reversal_values,
            "final_values": evaluation.entities[0].learner.values,
            "windows": {
                "before_first_100": summarize(&rows[0..100]),
                "before_last_100": summarize(&rows[100..200]),
                "after_first_100": summarize(&rows[200..300]),
                "after_last_100": summarize(&rows[300..400]),
            },
            "trials": rows,
        }));
    }
    serde_json::json!({
        "schema": "curious-experience-v1", "protocol": "water-competition-reversal-v1",
        "training_seed": training_seed, "evaluation_seed": evaluation_seed,
        "training_trials": TRAIN_TRIALS, "evaluation_trials_per_arm": EVAL_TRIALS,
        "reversal_at": REVERSAL, "controlled_body_reset": true,
        "training_summary": summarize(&training_rows), "training_values": trained.values,
        "arms": arms,
    })
}
fn main() {
    for (training_seed, evaluation_seed) in PAIRS {
        println!("{}", experiment(training_seed, evaluation_seed));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frozen_table_keeps_values_and_experience_counts_but_records_outcomes() {
        let mut world = laboratory(30001, Learner::default());
        for i in 0..20 {
            trial(&mut world, i, false);
        }
        world.entities[0].learner.update_enabled = false;
        let before = serde_json::to_value(&world.entities[0].learner.values).unwrap();
        let rows: Vec<_> = (0..20).map(|i| trial(&mut world, i, true)).collect();
        assert_eq!(
            before,
            serde_json::to_value(&world.entities[0].learner.values).unwrap()
        );
        assert!(rows.iter().any(|row| row.end.is_some()));
        assert!(rows.iter().all(|row| row.visits_before == row.visits_after));
    }
    #[test]
    fn body_reset_and_resolver_costs_are_explicit_and_equal() {
        let mut first = laboratory(30001, Learner::default());
        let mut second = laboratory(30001, Learner::default());
        first.entities[0].base.life_mut().age = 400;
        first.entities[0].base.life_mut().energy = 1;
        let a = trial(&mut first, 0, false);
        let b = trial(&mut second, 0, false);
        assert_eq!(
            serde_json::to_value(&a).unwrap(),
            serde_json::to_value(&b).unwrap()
        );
        // Reset occurs before World::tick; metabolism ages every actor before choice.
        assert_eq!(
            first.last_decisions[0].before.age,
            curious::rules::biology::AGING_RATE
        );
        assert_eq!(
            serde_json::to_value(&first.last_decisions[0].before).unwrap(),
            serde_json::to_value(&second.last_decisions[0].before).unwrap(),
        );
        assert_eq!(
            a.movement_energy,
            first.entities[0]
                .learner
                .last_episode
                .as_ref()
                .unwrap()
                .attempt
                .movement_energy,
        );
        assert_eq!(
            first.last_decisions[0].before.energy,
            second.last_decisions[0].before.energy
        );
    }
    #[test]
    fn protocol_is_bounded_and_training_evaluation_seeds_are_disjoint() {
        assert!(TRAIN_TRIALS + 4 * EVAL_TRIALS < 5000);
        assert!(
            PAIRS
                .iter()
                .all(|(_, evaluation)| PAIRS.iter().all(|(training, _)| training != evaluation))
        );
    }
}
