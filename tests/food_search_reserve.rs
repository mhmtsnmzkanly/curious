use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::{controller::EntityController, phase::EntityPhase, Entity},
    environment::{Environment, ResourceKind, ResourceSite},
    map::{cell::Cell, movement::Position, Map},
    random::Random,
    simulation::{checkpoint::State, SimulationEvent},
    world::{EntitySlot, World},
};

fn food_search_world(reserve_enabled: bool) -> World {
    let origin = Position::new(0, 0);
    let mut herbivore = HerbivoreEntity::default();
    let life = herbivore.life_mut();
    life.energy = 6;
    life.water = life.max_water;
    life.health = life.max_health;
    life.age = 0;
    life.max_age = 2_000;
    life.maturity_age = 2_000;
    life.speed = 3;

    let slot = EntitySlot::new(
        1,
        origin,
        EntityPhase::Active,
        EntityController::AI,
        Box::new(herbivore),
    );
    let mut world = World::with_seed(-6, 6, -3, 3, vec![slot], 7);
    world.map = Map::new(-6, 6, -3, 3);
    for y in world.map.min_y..=world.map.max_y {
        for x in world.map.min_x..=world.map.max_x {
            world.map.set_cell(Position::new(x, y), Cell::Empty);
        }
    }
    world.environment = Environment {
        sites: vec![ResourceSite {
            pos: origin,
            kind: ResourceKind::Food,
            capacity: 2,
            replenish: 2,
            interval: 2,
            schedule: None,
        }],
        ..Default::default()
    };
    world.policy.food_search_reserve_enabled = reserve_enabled;
    // Keep the rule-based exploratory direction paired across both worlds.
    world.rng = Random::new(7);
    world
}

#[test]
fn food_search_reserve_preserves_energy_for_next_tick_meal_approach() {
    let mut baseline = food_search_world(false);
    let mut candidate = food_search_world(true);
    let baseline_initial = State::capture(&baseline);
    let mut candidate_initial = State::capture(&candidate);
    candidate_initial.policy.food_search_reserve_enabled = false;
    assert_eq!(
        serde_json::to_value(baseline_initial).unwrap(),
        serde_json::to_value(candidate_initial).unwrap(),
        "paired worlds must differ only by the candidate flag"
    );

    let baseline_first = baseline.tick();
    let candidate_first = candidate.tick();
    let baseline_decision = &baseline.last_decisions[0].decision;
    let candidate_decision = &candidate.last_decisions[0].decision;
    assert_eq!(
        baseline_decision.reason,
        curious::entity::decision::Reason::SearchFood
    );
    assert_eq!(
        candidate_decision.reason,
        curious::entity::decision::Reason::FoodSearchReserve
    );
    assert_eq!(
        baseline_first
            .iter()
            .filter_map(|event| match event {
                SimulationEvent::EntityMoved {
                    id: 1, energy_cost, ..
                } => Some(*energy_cost),
                _ => None,
            })
            .sum::<usize>(),
        3
    );
    assert_eq!(
        candidate_first
            .iter()
            .filter_map(|event| match event {
                SimulationEvent::EntityMoved {
                    id: 1, energy_cost, ..
                } => Some(*energy_cost),
                _ => None,
            })
            .sum::<usize>(),
        2
    );

    let baseline_second = baseline.tick();
    let candidate_second = candidate.tick();
    assert!(!baseline_second
        .iter()
        .any(|event| matches!(event, SimulationEvent::EntityAte { id: 1, .. })));
    assert!(baseline_second.iter().any(|event| matches!(
        event,
        SimulationEvent::ActionRejected {
            id: 1,
            action,
            reason: curious::simulation::event::RejectionReason::OutOfRange,
            ..
        } if action == "Approach"
    )));
    assert!(candidate_second.iter().any(|event| matches!(
        event,
        SimulationEvent::EntityAte {
            id: 1,
            amount: 2,
            ..
        }
    )));
    assert!(candidate.entities[0].base.life().energy > 0);
}
