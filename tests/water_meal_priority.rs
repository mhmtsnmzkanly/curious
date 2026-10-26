use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::{
        Entity,
        learning::{Learner, WaterEnd},
        phase::EntityPhase,
    },
    map::{cell::Cell, movement::Position},
    simulation::SimulationEvent,
    world::{EntitySlot, World},
};

fn world(water_meal_priority_enabled: bool) -> World {
    let mut herbivore = HerbivoreEntity::default();
    let life = herbivore.life_mut();
    life.energy = 1;
    life.water = 1;
    life.age = 1;
    life.max_age = 2_000;
    life.maturity_age = 2_000;
    life.health = life.max_health;

    let slot = EntitySlot::new(
        1,
        Position::new(0, 0),
        EntityPhase::Active,
        curious::entity::controller::EntityController::AI,
        Box::new(herbivore),
    );
    let mut world = World::with_seed(-5, 5, -5, 5, vec![slot], 7);
    for y in world.map.min_y..=world.map.max_y {
        for x in world.map.min_x..=world.map.max_x {
            world.map.set_cell(Position::new(x, y), Cell::Empty);
        }
    }
    world.environment.sites.clear();
    world
        .map
        .set_cell(Position::new(0, 1), Cell::Food { amount: 3 });
    world
        .map
        .set_cell(Position::new(3, 0), Cell::Water { amount: 5 });
    world
        .map
        .set_cell(Position::new(-3, 0), Cell::Water { amount: 5 });
    world.policy.water_meal_priority_enabled = water_meal_priority_enabled;
    world.entities[0].learner = Learner {
        context_ttl_enabled: true,
        ..Learner::default()
    };
    world
}

#[test]
fn adjacent_meal_breaks_zero_payable_water_lock_then_later_drink_is_paid() {
    let mut baseline = world(false);
    let mut candidate = world(true);

    let baseline_tick1 = baseline.tick();
    let candidate_tick1 = candidate.tick();

    let baseline_trace = serde_json::to_value(&baseline.last_decisions[0]).unwrap();
    assert!(baseline_trace.get("water_meal_override").is_none());
    let _: curious::entity::decision::DecisionTrace =
        serde_json::from_value(baseline_trace).unwrap();

    assert!(!baseline_tick1.iter().any(|event| matches!(
        event,
        SimulationEvent::EntityAte { id: 1, .. } | SimulationEvent::EntityDrank { id: 1, .. }
    )));
    assert_eq!(
        baseline.last_decisions[0].decision.reason,
        curious::entity::decision::Reason::LearnedWater
    );
    assert!(matches!(
        &baseline.last_decisions[0].decision.intent,
        curious::entity::intent::Intent::Drink { .. }
    ));
    assert!(candidate_tick1.iter().any(|event| matches!(
        event,
        SimulationEvent::EntityAte {
            id: 1,
            amount: 3,
            corpse_id: None,
            ..
        }
    )));
    assert_eq!(
        candidate.last_decisions[0].decision.reason,
        curious::entity::decision::Reason::EatBeforeWater
    );
    assert!(matches!(
        &candidate.last_decisions[0].decision.intent,
        curious::entity::intent::Intent::Eat { .. }
    ));
    assert!(
        candidate.last_decisions[0]
            .water_meal_override
            .as_ref()
            .is_some_and(|e| {
                matches!(
                    &e.replaced_decision.intent,
                    curious::entity::intent::Intent::Drink { .. }
                ) && e.visible_water_amount > 0
                    && e.visible_plant_food_amount > 0
            })
    );
    assert!(
        !candidate_tick1
            .iter()
            .any(|event| matches!(event, SimulationEvent::EntityDrank { id: 1, .. }))
    );

    let candidate_slot = &candidate.entities[0];
    assert_eq!(candidate_slot.base.life().energy, 3);
    let interrupted = candidate_slot.learner.last_episode.as_ref().unwrap();
    assert_eq!(interrupted.end, WaterEnd::Interrupted);
    assert_eq!(interrupted.reward, None);
    assert!(candidate_slot.learner.episode.is_none());
    assert!(candidate_slot.learner.values.is_empty());

    let baseline_tick2 = baseline.tick();
    let candidate_tick2 = candidate.tick();
    assert!(
        !baseline_tick2
            .iter()
            .any(|event| matches!(event, SimulationEvent::EntityDrank { id: 1, .. }))
    );
    assert!(candidate_tick2.iter().any(|event| matches!(
        event,
        SimulationEvent::EntityDrank {
            id: 1,
            amount,
            ..
        } if *amount > 0
    )));
    assert!(candidate_tick2.iter().any(|event| matches!(
        event,
        SimulationEvent::EntityMoved {
            id: 1,
            energy_cost: 2,
            ..
        }
    )));
    assert_eq!(candidate.entities[0].base.life().energy, 0);
    assert!(candidate.entities[0].base.life().water > 0);
    assert_eq!(
        candidate.last_decisions[0].decision.reason,
        curious::entity::decision::Reason::LearnedWater
    );
    assert!(
        candidate.entities[0]
            .learner
            .last_episode
            .as_ref()
            .is_some_and(|r| r.end == WaterEnd::Reached)
    );
}
