//! Physical world integration: learning cannot mint food, bypass priorities, or
//! change the continuation represented by a checkpoint.
use curious::{
    creatures::{carnivore::CarnivoreEntity, herbivore::HerbivoreEntity},
    entity::{
        controller::EntityController,
        decision::Reason,
        hunt_learning::{HuntContext, HuntEnd, HuntValue},
        intent::Intent,
        phase::EntityPhase,
    },
    logger::Logger,
    map::{cell::Cell, movement::Position, Map},
    simulation::{checkpoint::State, SimulationEvent},
    world::{EntitySlot, World},
};
fn hunt_world(prey_health: usize) -> World {
    let mut predator = CarnivoreEntity::default();
    predator.life_state.energy = 40; // hungry, hydrated, able to pay a real attack
    predator.life_state.speed = 1;
    let mut prey = HerbivoreEntity::default();
    prey.life_state.health = prey_health;
    prey.life_state.energy = 30; // avoid passive healing obscuring the one-hit fixture
    let mut hunter = EntitySlot::new(
        1,
        Position::new(1, 1),
        EntityPhase::Active,
        EntityController::AI,
        Box::new(predator),
    );
    hunter.hunt_learner.enabled = true;
    bare_world(vec![
        hunter,
        EntitySlot::new(
            10,
            Position::new(2, 1),
            EntityPhase::Active,
            EntityController::Player(10),
            Box::new(prey),
        ),
    ])
}
fn bare_world(entities: Vec<EntitySlot>) -> World {
    let mut w = World::with_seed(0, 5, 0, 5, entities, 50);
    w.logger = Logger::disabled();
    w.map = Map::new(0, 5, 0, 5);
    w.environment = Default::default();
    w
}
fn reason(w: &World, id: usize) -> Reason {
    w.last_decisions
        .iter()
        .find(|d| d.id == id)
        .unwrap()
        .decision
        .reason
}
fn knowledge() -> HuntValue {
    HuntValue {
        context: HuntContext {
            energy: 1,
            stamina: 2,
            distance: 0,
            health: 1,
        },
        value: 70,
        visits: 4,
    }
}
#[test]
fn own_physical_kill_then_real_meal_produces_audited_reward() {
    let mut w = hunt_world(1);
    let attack_events = w.tick();
    assert_eq!(reason(&w, 1), Reason::LearnedHunt);
    let cost = attack_events
        .iter()
        .find_map(|e| match e {
            SimulationEvent::EntityAttacked {
                attacker: 1,
                target: 10,
                energy_cost,
                killed: true,
                ..
            } => Some(*energy_cost),
            _ => None,
        })
        .expect("resolver must identify the actual killing blow and paid cost");
    assert_eq!(cost, 3);
    let hunter = w.entities.iter().find(|s| s.id == 1).unwrap();
    assert!(hunter.hunt_learner.values.is_empty()); // killing is not nutrition
    assert!(hunter.hunt_learner.episode.as_ref().unwrap().own_kill);
    assert!(w
        .entities
        .iter()
        .find(|s| s.id == 10)
        .unwrap()
        .phase
        .is_corpse());
    let energy_before = hunter.base.life().energy;
    let food_events = w.tick();
    assert_eq!(reason(&w, 1), Reason::SeekFood); // learner preserves the food policy
    let amount = food_events
        .iter()
        .find_map(|e| match e {
            SimulationEvent::EntityAte {
                id: 1,
                corpse_id: Some(10),
                amount,
                ..
            } => Some(*amount),
            _ => None,
        })
        .expect("food reward requires an actual meal from the identified target");
    let hunter = w.entities.iter().find(|s| s.id == 1).unwrap();
    assert_eq!(hunter.base.life().energy, energy_before - 1 + amount);
    let result = hunter.hunt_learner.last_episode.as_ref().unwrap();
    assert_eq!(result.end, HuntEnd::Fed);
    assert_eq!(result.attempt.ticks, 2);
    assert_eq!(result.attempt.attack_energy, cost);
    assert_eq!(result.attempt.movement_energy, 0);
    assert_eq!(result.attempt.food, amount);
    assert_eq!(result.reward, Some((amount * 10 - cost * 2 - 2) as i32));
    assert_eq!(hunter.hunt_learner.values.len(), 1);
}
#[test]
fn pending_own_kill_checkpoint_resumes_all_state_and_rng_identically() {
    let mut original = hunt_world(1);
    original.tick();
    let state = State::capture(&original);
    assert_eq!(state.version, 13);
    assert!(
        state
            .entities
            .iter()
            .find(|s| s.id == 1)
            .unwrap()
            .hunt_learner
            .episode
            .as_ref()
            .unwrap()
            .own_kill
    );
    let json = serde_json::to_string(&state).unwrap();
    let mut restored = serde_json::from_str::<State>(&json)
        .unwrap()
        .restore()
        .unwrap();
    for _ in 0..8 {
        assert_eq!(
            serde_json::to_value(original.tick()).unwrap(),
            serde_json::to_value(restored.tick()).unwrap()
        );
        assert_eq!(
            serde_json::to_value(State::capture(&original)).unwrap(),
            serde_json::to_value(State::capture(&restored)).unwrap()
        );
        assert_eq!(original.rng.state(), restored.rng.state());
    }
}
#[test]
fn another_predators_real_kill_has_no_credit_for_the_pursuer() {
    let mut w = hunt_world(1);
    // Pursuer must walk this tick while a different actor attacks the same prey.
    w.entities.iter_mut().find(|s| s.id == 1).unwrap().pos = Position::new(0, 1);
    w.entities.push(EntitySlot::new(
        2,
        Position::new(2, 2),
        EntityPhase::Active,
        EntityController::Player(2),
        Box::new(CarnivoreEntity::default()),
    ));
    w.next_entity_id = 11;
    w.push_command(2, Intent::Attack { target_id: 10 });
    let events = w.tick();
    assert_eq!(reason(&w, 1), Reason::LearnedHunt);
    assert!(events.iter().any(|e| matches!(
        e,
        SimulationEvent::EntityAttacked {
            attacker: 2,
            target: 10,
            killed: true,
            ..
        }
    )));
    let learner = &w.entities.iter().find(|s| s.id == 1).unwrap().hunt_learner;
    let result = learner.last_episode.as_ref().unwrap();
    assert_eq!(result.end, HuntEnd::ForeignKill);
    assert_eq!(result.reward, None);
    assert_eq!(result.attempt.food, 0);
    assert!(!result.attempt.own_kill);
    assert!(learner.values.is_empty());
}

#[test]
fn resolver_attributes_current_prey_defense_and_sans_censors_third_party_damage() {
    let mut own = hunt_world(100);
    own.push_command(10, Intent::Attack { target_id: 1 });
    let first = own.tick();
    assert!(first.iter().any(|e| matches!(
        e,
        SimulationEvent::EntityAttacked { attacker: 10, target: 1, damage, .. } if *damage > 0
    )));
    let episode = own
        .entities
        .iter()
        .find(|s| s.id == 1)
        .unwrap()
        .hunt_learner
        .episode
        .as_ref()
        .unwrap();
    assert!(episode.incoming_damage > 0);
    assert!(
        !episode.ambiguous,
        "damage from the current prey is attributable"
    );

    let mut mixed = hunt_world(100);
    mixed.entities.push(EntitySlot::new(
        2,
        Position::new(1, 2),
        EntityPhase::Active,
        EntityController::Player(2),
        Box::new(CarnivoreEntity::default()),
    ));
    mixed.next_entity_id = 11;
    mixed.push_command(10, Intent::Attack { target_id: 1 });
    mixed.push_command(2, Intent::Attack { target_id: 1 });
    let events = mixed.tick();
    assert!(events.iter().any(|e| matches!(
        e,
        SimulationEvent::EntityAttacked { attacker: 10, target: 1, damage, .. } if *damage > 0
    )));
    assert!(events.iter().any(|e| matches!(
        e,
        SimulationEvent::EntityAttacked { attacker: 2, target: 1, damage, .. } if *damage > 0
    )));
    let episode = mixed
        .entities
        .iter()
        .find(|s| s.id == 1)
        .unwrap()
        .hunt_learner
        .episode
        .as_ref()
        .unwrap();
    assert!(episode.incoming_damage > 0);
    assert!(
        episode.ambiguous,
        "third-party damage remains censored alongside prey defense"
    );
}
#[test]
fn external_command_and_real_thirst_interrupt_without_becoming_bad_examples() {
    for external in [true, false] {
        let mut w = hunt_world(100);
        w.tick();
        assert!(w
            .entities
            .iter()
            .find(|s| s.id == 1)
            .unwrap()
            .hunt_learner
            .episode
            .is_some());
        if external {
            w.push_command(1, Intent::Idle { duration: 1 });
        } else {
            w.entities
                .iter_mut()
                .find(|s| s.id == 1)
                .unwrap()
                .base
                .life_mut()
                .water = 10;
            w.map
                .set_cell(Position::new(1, 2), Cell::Water { amount: 20 });
        }
        w.tick();
        assert_eq!(
            reason(&w, 1),
            if external {
                Reason::ExternalCommand
            } else {
                Reason::SeekWater
            }
        );
        let learner = &w.entities.iter().find(|s| s.id == 1).unwrap().hunt_learner;
        assert!(learner.episode.is_none());
        assert!(learner.values.is_empty());
        let result = learner.last_episode.as_ref().unwrap();
        assert_eq!(result.end, HuntEnd::Interrupted);
        assert_eq!(result.reward, None);
    }
}
#[test]
fn physical_birth_inherits_mode_flags_but_never_experience() {
    for second_enabled in [true, false] {
        let parents = (1..=2)
            .map(|id| {
                let mut body = HerbivoreEntity::default();
                body.life_state.age = 40;
                let mut slot = EntitySlot::new(
                    id,
                    Position::new(id as isize, 1),
                    EntityPhase::Active,
                    EntityController::Player(id as u32),
                    Box::new(body),
                );
                slot.hunt_learner.enabled = id == 1 || second_enabled;
                slot.hunt_learner.update_enabled = id == 1;
                slot.hunt_learner.values.push(knowledge());
                slot
            })
            .collect();
        let mut w = bare_world(parents);
        w.push_command(1, Intent::Mate { target_id: 2 });
        let events = w.tick();
        assert!(events
            .iter()
            .any(|e| matches!(e, SimulationEvent::EntityBorn { .. })));
        let child = w.entities.iter().find(|s| s.generation == 1).unwrap();
        assert_eq!(child.hunt_learner.enabled, second_enabled);
        assert!(!child.hunt_learner.update_enabled);
        assert!(child.hunt_learner.values.is_empty());
        assert!(child.hunt_learner.episode.is_none());
        assert!(child.hunt_learner.last_episode.is_none());
    }
}
#[test]
fn legacy_migration_defaults_and_current_hunt_validation_are_enforced() {
    let w = hunt_world(1);
    let mut old = serde_json::to_value(State::capture(&w)).unwrap();
    old["version"] = 12.into();
    for individual in old["entities"].as_array_mut().unwrap() {
        individual.as_object_mut().unwrap().remove("hunt_learner");
        individual["learner"]
            .as_object_mut()
            .unwrap()
            .remove("update_enabled");
    }
    let migrated = serde_json::from_value::<State>(old)
        .unwrap()
        .restore()
        .unwrap();
    for individual in migrated.entities {
        assert!(!individual.hunt_learner.enabled);
        assert!(individual.learner.update_enabled);
    }
    let mut invalid = State::capture(&w);
    invalid.entities[0].hunt_learner.values.push(knowledge());
    invalid.entities[0].hunt_learner.values[0].context.health = 4;
    assert!(invalid.restore().is_err());
    let mut duplicate = State::capture(&w);
    duplicate.entities[0].hunt_learner.values = vec![knowledge(), knowledge()];
    assert!(duplicate.restore().is_err());
}
