//! Bounded preregistered environment-cycle and paid-trait comparison.
//! Protocol: docs/ECOLOGY_TRAITS_PROTOCOL.md.
use curious::{
    entity::species::Species,
    environment::{Environment, ResourceKind, ResourceSchedule, ResourceSite},
    map::{cell::Cell, movement::Position, Map},
    scenario::Scenario,
    simulation::{checkpoint::State, SimulationEvent},
    world::World,
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    io::{self, Write},
};

const EVALUATION_SEEDS: [u64; 8] = [50001, 50002, 50003, 50004, 50005, 50006, 50007, 50008];
const CALIBRATION_SEEDS: [u64; 4] = [50041, 50042, 50043, 50044];
const TICKS: usize = 2_000;

#[derive(Clone, Copy, Debug, Serialize)]
struct Arm {
    name: &'static str,
    mobility: u8,
    vision: u8,
    stamina: u8,
}

const ARMS: [Arm; 5] = [
    Arm {
        name: "control",
        mobility: 0,
        vision: 0,
        stamina: 0,
    },
    Arm {
        name: "mobility",
        mobility: 1,
        vision: 0,
        stamina: 0,
    },
    Arm {
        name: "vision",
        mobility: 0,
        vision: 1,
        stamina: 0,
    },
    Arm {
        name: "stamina",
        mobility: 0,
        vision: 0,
        stamina: 1,
    },
    Arm {
        name: "vision_stamina",
        mobility: 0,
        vision: 1,
        stamina: 1,
    },
];

fn emit<T: Serialize>(out: &mut impl Write, value: &T) -> curious::Result<()> {
    serde_json::to_writer(&mut *out, value)?;
    out.write_all(b"\n")?;
    out.flush()?;
    Ok(())
}

fn site(pos: Position, schedule: Option<ResourceSchedule>) -> ResourceSite {
    ResourceSite {
        pos,
        kind: ResourceKind::Food,
        capacity: 100,
        replenish: 1,
        interval: 1,
        schedule,
    }
}

fn mechanism_check(out: &mut impl Write) -> curious::Result<()> {
    let p0 = Position::new(0, 0);
    let p1 = Position::new(1, 0);
    let schedule = ResourceSchedule {
        period: 12,
        active_ticks: 4,
        geographic_phase_step: 1,
        phase_offset: 0,
    };
    let mut world = World::with_seed(-2, 2, -2, 2, vec![], 87001);
    world.map = Map::new(-2, 2, -2, 2);
    world.environment = Environment {
        sites: vec![site(p0, Some(schedule)), site(p1, Some(schedule))],
        ..Default::default()
    };
    world.map.set_cell(p0, Cell::Food { amount: 0 });
    world.map.set_cell(p1, Cell::Food { amount: 0 });
    let mut closed_control = World::with_seed(-2, 2, -2, 2, vec![], 87001);
    closed_control.map = Map::new(-2, 2, -2, 2);
    closed_control.environment = Environment {
        sites: vec![site(p0, None), site(p1, None)],
        ..Default::default()
    };
    closed_control.map.set_cell(p0, Cell::Food { amount: 0 });
    closed_control.map.set_cell(p1, Cell::Food { amount: 0 });
    let initial_rng = world.rng.state();
    let mut events_seen = 0usize;
    let mut amounts = [0usize; 2];
    for tick in 1..=48 {
        let events = world.tick();
        let control_events = closed_control.tick();
        if control_events
            .iter()
            .filter(|e| matches!(e, SimulationEvent::ResourceReplenished { .. }))
            .count()
            != 2
        {
            return Err(format!("closed schedule control failed at tick {tick}").into());
        }
        let active = [
            (tick as isize).rem_euclid(12) < 4,
            (tick as isize + 1).rem_euclid(12) < 4,
        ];
        for i in 0..2 {
            amounts[i] += usize::from(active[i]);
        }
        let replenished = events
            .iter()
            .filter(|e| matches!(e, SimulationEvent::ResourceReplenished { .. }))
            .count();
        if replenished != usize::from(active[0]) + usize::from(active[1]) {
            return Err(format!("unexpected scheduled resource events at tick {tick}").into());
        }
        events_seen += replenished;
        if world.map.cell(p0) != Some(&Cell::Food { amount: amounts[0] })
            || world.map.cell(p1) != Some(&Cell::Food { amount: amounts[1] })
            || world.rng.state() != initial_rng
        {
            return Err(format!("schedule phase or RNG mismatch at tick {tick}").into());
        }
        if tick == 17 {
            let mut resumed = State::capture(&world).restore()?;
            for continuation_tick in 18..=48 {
                let continuation_events = world.tick();
                let resumed_events = resumed.tick();
                let control_events = closed_control.tick();
                let active = [
                    (continuation_tick as isize).rem_euclid(12) < 4,
                    (continuation_tick as isize + 1).rem_euclid(12) < 4,
                ];
                for i in 0..2 {
                    amounts[i] += usize::from(active[i]);
                }
                let expected_events = usize::from(active[0]) + usize::from(active[1]);
                let count_events = |events: &[SimulationEvent]| {
                    events
                        .iter()
                        .filter(|e| matches!(e, SimulationEvent::ResourceReplenished { .. }))
                        .count()
                };
                if count_events(&continuation_events) != expected_events
                    || count_events(&resumed_events) != expected_events
                    || count_events(&control_events) != 2
                    || world.map.cell(p0) != Some(&Cell::Food { amount: amounts[0] })
                    || world.map.cell(p1) != Some(&Cell::Food { amount: amounts[1] })
                {
                    return Err(format!(
                        "checkpointed schedule phase mismatch at tick {continuation_tick}"
                    )
                    .into());
                }
                events_seen += expected_events;
                if serde_json::to_value(State::capture(&world))?
                    != serde_json::to_value(State::capture(&resumed))?
                {
                    return Err("scheduled checkpoint continuation mismatch".into());
                }
            }
            if world.rng.state() != resumed.rng.state() {
                return Err("scheduled checkpoint RNG mismatch".into());
            }
            emit(
                out,
                &serde_json::json!({
                    "kind":"mechanism_check", "status":"passed", "ticks":48,
                    "checkpoint_tick":17, "resource_events":events_seen,
                    "rng_unchanged":true, "state_rng_continuation_equal":true,
                    "schedule":{"period":12,"active_ticks":4,"geographic_phase_step":1,"phase_offset":0}
                }),
            )?;
            return Ok(());
        }
    }
    Err("mechanism check did not reach checkpoint".into())
}

fn resource_totals(world: &World) -> (usize, usize) {
    let mut food = 0;
    let mut water = 0;
    for y in world.map.min_y..=world.map.max_y {
        for x in world.map.min_x..=world.map.max_x {
            match world.map.cell(Position::new(x, y)) {
                Some(Cell::Food { amount }) => food += amount,
                Some(Cell::Water { amount }) => water += amount,
                _ => {}
            }
        }
    }
    (food, water)
}

fn run_world(seed: u64, arm: Arm, cyclic: bool, out: &mut impl Write) -> curious::Result<()> {
    let source = include_str!("../scenarios/ecology-traits-lab.json");
    let mut scenario: Scenario = serde_json::from_str(source)?;
    for founder in &mut scenario.founders {
        founder.genome.mobility = arm.mobility;
        founder.genome.vision = arm.vision;
        founder.genome.stamina = arm.stamina;
    }
    scenario.genetics.enabled = true;
    scenario.genetics.mutation_per_mille = 0;
    scenario.genetics.extra_traits_enabled = true;
    if cyclic {
        for site in &mut scenario.sites {
            site.schedule = Some(ResourceSchedule {
                period: 12,
                active_ticks: 4,
                geographic_phase_step: 1,
                phase_offset: 0,
            });
        }
    } else {
        for site in &mut scenario.sites {
            site.schedule = None;
        }
    }
    let mut world = scenario.build(seed)?;
    let mut total_births = 0usize;
    let mut f1_births = 0usize;
    let mut f2_births = 0usize;
    let mut food_eaten = 0usize;
    let mut movement_energy = 0usize;
    let mut requested_vision = 0usize;
    let mut requested_stamina = 0usize;
    let mut paid_vision = 0usize;
    let mut paid_stamina = 0usize;
    let mut resource_events = 0usize;
    let mut death_causes = BTreeMap::<String, usize>::new();
    let mut f1_breeders = BTreeSet::<usize>::new();
    let mut descendant_breeders = BTreeSet::<usize>::new();
    let mut event_rows = Vec::with_capacity(TICKS);
    let mut samples = Vec::new();

    for tick in 1..=TICKS {
        let energy_before: BTreeMap<usize, usize> = world
            .entities
            .iter()
            .map(|entity| (entity.id, entity.base.life().energy))
            .collect();
        let events = world.tick();
        let mut row = serde_json::json!({"tick":tick,"births":0,"deaths":0,"eats":0,"metabolized":0,"resource_replenishments":0});
        let mut tick_paid = BTreeMap::<usize, usize>::new();
        for event in &events {
            match event {
                SimulationEvent::EntityBorn { id, .. } => {
                    total_births += 1;
                    row["births"] = (row["births"].as_u64().unwrap() + 1).into();
                    let generation = world
                        .entities
                        .iter()
                        .find(|e| e.id == *id)
                        .map_or(0, |e| e.generation);
                    match generation {
                        1 => f1_births += 1,
                        2 => f2_births += 1,
                        _ => {}
                    }
                }
                SimulationEvent::EntityMated {
                    parent1, parent2, ..
                } => {
                    for id in [parent1, parent2] {
                        if let Some(parent) = world.entities.iter().find(|e| e.id == *id) {
                            if parent.generation == 1 {
                                f1_breeders.insert(*id);
                            }
                            if parent.generation > 0 {
                                descendant_breeders.insert(*id);
                            }
                        }
                    }
                }
                SimulationEvent::EntityDied { cause, .. } => {
                    row["deaths"] = (row["deaths"].as_u64().unwrap() + 1).into();
                    *death_causes.entry(format!("{cause:?}")).or_default() += 1;
                }
                SimulationEvent::EntityAte { amount, .. } => {
                    row["eats"] = (row["eats"].as_u64().unwrap() + 1).into();
                    food_eaten += amount;
                }
                SimulationEvent::EntityMoved { energy_cost, .. } => movement_energy += energy_cost,
                SimulationEvent::EntityMetabolized {
                    id,
                    vision_cost,
                    stamina_cost,
                    vision_paid,
                    stamina_paid,
                } => {
                    row["metabolized"] = (row["metabolized"].as_u64().unwrap() + 1).into();
                    requested_vision += vision_cost;
                    requested_stamina += stamina_cost;
                    paid_vision += vision_paid;
                    paid_stamina += stamina_paid;
                    if vision_paid > vision_cost || stamina_paid > stamina_cost {
                        return Err("metabolism audit exceeds requested cost".into());
                    }
                    let paid = vision_paid.saturating_add(*stamina_paid);
                    *tick_paid.entry(*id).or_default() += paid;
                }
                SimulationEvent::ResourceReplenished { .. } => {
                    row["resource_replenishments"] =
                        (row["resource_replenishments"].as_u64().unwrap() + 1).into();
                    resource_events += 1;
                }
                _ => {}
            }
        }
        for (id, paid) in tick_paid {
            let available = energy_before
                .get(&id)
                .copied()
                .unwrap_or(0)
                .saturating_sub(curious::rules::biology::BMR_ENERGY_COST);
            if paid > available {
                return Err(format!(
                    "trait energy payment exceeds post-basal energy for entity {id} at tick {tick}"
                )
                .into());
            }
        }
        event_rows.push(row);
        if tick == 1 || tick % 100 == 0 || tick == TICKS {
            let alive = world
                .entities
                .iter()
                .filter(|e| {
                    e.base.life().is_alive()
                        && matches!(
                            &e.phase,
                            curious::entity::phase::EntityPhase::Active
                                | curious::entity::phase::EntityPhase::Sleeping { .. }
                        )
                })
                .count();
            let (food, water) = resource_totals(&world);
            samples.push(serde_json::json!({"tick":tick,"alive":alive,"population":world.entities.len(),"food":food,"water":water}));
        }
    }

    let mut genotype_counts = BTreeMap::<String, usize>::new();
    for entity in &world.entities {
        if entity.base.life().is_alive()
            && matches!(
                &entity.phase,
                curious::entity::phase::EntityPhase::Active
                    | curious::entity::phase::EntityPhase::Sleeping { .. }
            )
        {
            *genotype_counts
                .entry(format!(
                    "{}:{}:{}",
                    entity.genome.mobility, entity.genome.vision, entity.genome.stamina
                ))
                .or_default() += 1;
        }
    }
    let living_founders = world
        .entities
        .iter()
        .filter(|e| {
            e.id <= 8
                && e.base.species() == Species::Herbivore
                && e.base.life().is_alive()
                && matches!(
                    &e.phase,
                    curious::entity::phase::EntityPhase::Active
                        | curious::entity::phase::EntityPhase::Sleeping { .. }
                )
        })
        .count();
    let living_f1_breeders = f1_breeders
        .iter()
        .filter(|id| {
            world.entities.iter().any(|e| {
                e.id == **id
                    && e.base.life().is_alive()
                    && matches!(
                        &e.phase,
                        curious::entity::phase::EntityPhase::Active
                            | curious::entity::phase::EntityPhase::Sleeping { .. }
                    )
            })
        })
        .count();
    let living_descendant_breeders = descendant_breeders
        .iter()
        .filter(|id| {
            world.entities.iter().any(|e| {
                e.id == **id
                    && e.base.life().is_alive()
                    && matches!(
                        &e.phase,
                        curious::entity::phase::EntityPhase::Active
                            | curious::entity::phase::EntityPhase::Sleeping { .. }
                    )
            })
        })
        .count();
    let final_state = State::capture(&world);
    emit(
        out,
        &serde_json::json!({
            "kind":"replicate", "seed":seed, "arm":arm, "environment":if cyclic {"cycle"} else {"static"},
            "ticks":TICKS, "extra_traits_enabled":world.genetics.extra_traits_enabled,
            "genetics_enabled":world.genetics.enabled, "mutation_per_mille":world.genetics.mutation_per_mille,
            "living_founders":living_founders, "living_f1_breeders":living_f1_breeders,
            "f1_offspring_that_bred":f1_breeders.len(),
            "descendant_breeders_that_bred":descendant_breeders.len(),
            "living_descendant_breeders":living_descendant_breeders,
            "total_births":total_births, "f1_births":f1_births, "f2_births":f2_births,
            "max_generation":world.entities.iter().map(|e| e.generation).max().unwrap_or(0),
            "genotype_counts":genotype_counts, "death_causes":death_causes,
            "food_eaten":food_eaten, "movement_energy":movement_energy,
            "requested_vision_maintenance":requested_vision, "requested_stamina_maintenance":requested_stamina,
            "paid_vision_maintenance":paid_vision, "paid_stamina_maintenance":paid_stamina,
            "resource_replenishment_events":resource_events,
            "samples":samples, "event_rows":event_rows, "final_state":final_state
        }),
    )?;
    Ok(())
}

fn main() -> curious::Result<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 || !["calibration", "evaluation"].contains(&args[1].as_str()) {
        return Err("usage: ecology_traits_lab calibration|evaluation".into());
    }
    let (phase, seeds): (&str, &[u64]) = if args[1] == "calibration" {
        ("calibration", &CALIBRATION_SEEDS)
    } else {
        ("evaluation", &EVALUATION_SEEDS)
    };
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    emit(
        &mut out,
        &serde_json::json!({"kind":"run","schema":"ecology-traits-lab-v1","phase":phase,"ticks_per_world":TICKS,"seeds":seeds}),
    )?;
    mechanism_check(&mut out)?;
    for seed in seeds {
        for arm in ARMS {
            for cyclic in [false, true] {
                run_world(*seed, arm, cyclic, &mut out)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preregistered_schedule_trace_and_checkpoint_continuation_pass() {
        let mut output = Vec::new();
        mechanism_check(&mut output).unwrap();
        let result: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(result["kind"], "mechanism_check");
        assert_eq!(result["status"], "passed");
        assert_eq!(result["ticks"], 48);
        assert_eq!(result["rng_unchanged"], true);
    }
}
