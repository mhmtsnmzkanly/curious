//! Paired food-search reserve screen. One registered World per invocation.
//! See docs/FOOD_SEARCH_RESERVE_GUARD_V1_PROTOCOL_20261026.md.
use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::{controller::EntityController, phase::EntityPhase},
    environment::{Environment, ResourceKind, ResourceSite},
    logger::Logger,
    map::{Map, cell::Cell, movement::Position},
    scenario::Scenario,
    simulation::{SimulationEvent, checkpoint::State},
    world::{EntitySlot, World},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
};

const TICKS: usize = 300;
const MAX_IDENTITIES: usize = 10_000;
const MAX_EVENTS_PER_TICK: usize = 20_000;
const MAX_WORLD_OUTPUT: u64 = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    Baseline,
    Candidate,
}
impl Arm {
    fn parse(s: &str) -> Result<Self, String> {
        match s {
            "baseline" => Ok(Self::Baseline),
            "candidate" => Ok(Self::Candidate),
            _ => Err(format!("unknown arm {s}")),
        }
    }
}
fn pos(x: isize, y: isize) -> Position {
    Position::new(x, y)
}
fn site(
    pos: Position,
    kind: ResourceKind,
    capacity: usize,
    replenish: usize,
    interval: usize,
) -> ResourceSite {
    ResourceSite {
        pos,
        kind,
        capacity,
        replenish,
        interval,
        schedule: None,
    }
}

fn v1_world(seed: u64, candidate: bool) -> World {
    let mut body = HerbivoreEntity::default();
    body.life_state.max_age = 2000;
    body.life_state.maturity_age = 2000;
    let actor = EntitySlot::new(
        1,
        pos(0, 0),
        EntityPhase::Active,
        EntityController::AI,
        Box::new(body),
    );
    let mut world = World::with_seed(-6, 6, -3, 3, vec![actor], seed);
    world.logger = Logger::disabled();
    world.map = Map::new(-6, 6, -3, 3);
    world.map.set_cell(pos(0, -1), Cell::Food { amount: 30 });
    world.map.set_cell(pos(-2, 0), Cell::Water { amount: 12 });
    world.map.set_cell(pos(4, 0), Cell::Water { amount: 12 });
    world.environment = Environment {
        sites: vec![
            site(pos(-2, 0), ResourceKind::Water, 12, 4, 2),
            site(pos(4, 0), ResourceKind::Water, 12, 1, 12),
            site(pos(0, -1), ResourceKind::Food, 30, 2, 2),
        ],
        ..Default::default()
    };
    world.observation.full_for = vec![1];
    world.entities[0].learner.enabled = true;
    world.entities[0].learner.update_enabled = true;
    world.entities[0].learner.context_ttl_enabled = true;
    world.genetics.enabled = true;
    world.genetics.mutation_per_mille = 0;
    world.genetics.extra_traits_enabled = false;
    world.policy.food_search_reserve_enabled = candidate;
    world
}

fn scenario_world(path: &str, seed: u64, candidate: bool) -> curious::Result<World> {
    let bytes = fs::read(path)?;
    let mut scenario: Scenario = serde_json::from_slice(&bytes)?;
    scenario.genetics.enabled = true;
    scenario.genetics.mutation_per_mille = 0;
    scenario.genetics.extra_traits_enabled = false;
    scenario.policy.food_search_reserve_enabled = candidate;
    scenario.policy.mate_rendezvous = false;
    scenario.policy.reserve_opportunities = false;
    scenario.policy.water_residual_enabled = false;
    scenario.policy.injury_capacity_penalty = false;
    scenario.policy.juvenile_care_enabled = false;
    scenario.policy.ambush_predation_enabled = false;
    scenario.policy.local_coordination_enabled = false;
    let mut world = scenario.build(seed)?;
    world.observation.full_for = vec![1];
    Ok(world)
}

fn alive(e: &EntitySlot) -> bool {
    e.base.life().is_alive()
        && matches!(e.phase, EntityPhase::Active | EntityPhase::Sleeping { .. })
}
fn alive_counts(world: &World) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for e in world.entities.iter().filter(|e| alive(e)) {
        *out.entry(format!("{:?}", e.base.species())).or_default() += 1;
    }
    out
}
fn resource_totals(world: &World) -> (usize, usize) {
    let mut food = 0;
    let mut water = 0;
    for y in world.map.min_y..=world.map.max_y {
        for x in world.map.min_x..=world.map.max_x {
            match world.map.cell(pos(x, y)) {
                Some(Cell::Food { amount }) => food += amount,
                Some(Cell::Water { amount }) => water += amount,
                _ => {}
            }
        }
    }
    (food, water)
}
fn write_row(
    writer: &mut BufWriter<std::fs::File>,
    size: &mut u64,
    value: &Value,
) -> curious::Result<()> {
    let bytes = serde_json::to_vec(value)?;
    let next = size
        .checked_add(bytes.len() as u64 + 1)
        .ok_or("output size overflow")?;
    if next > MAX_WORLD_OUTPUT {
        return Err("per-world output cap exceeded".into());
    }
    writer.write_all(&bytes)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    *size = next;
    Ok(())
}

fn run() -> curious::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 6 {
        return Err(
            "usage: food_search_reserve_lab SCENARIO_OR_v1 SEED ARM HABITAT OUTPUT.jsonl".into(),
        );
    }
    let seed: u64 = args[2].parse()?;
    let arm = Arm::parse(&args[3])?;
    let habitat = &args[4];
    let candidate = arm == Arm::Candidate;
    let mut world = if args[1] == "v1" {
        v1_world(seed, candidate)
    } else {
        scenario_world(&args[1], seed, candidate)?
    };
    let scenario_name = if args[1] == "v1" {
        "water-adaptation-v1-scarce-food".to_owned()
    } else {
        habitat.to_owned()
    };
    let founder_ids: BTreeSet<usize> = world.entities.iter().map(|e| e.id).collect();
    let initial_state = serde_json::to_value(State::capture(&world))?;
    let path = Path::new(&args[5]);
    let file = OpenOptions::new().write(true).create_new(true).open(path)?;
    let mut writer = BufWriter::new(file);
    let mut output_size = 0;
    write_row(
        &mut writer,
        &mut output_size,
        &json!({"type":"header","schema":"curious-food-search-reserve-v1-jsonl","habitat":habitat,"scenario_name":scenario_name,"scenario_source":if args[1]=="v1" {Value::Null} else {json!(args[1])},"seed":seed,"arm":arm_name(arm),"ticks":TICKS,"candidate_policy":{"food_search_reserve_enabled":candidate},"other_policy":{"mate_rendezvous":false,"reserve_opportunities":false,"water_residual_enabled":false,"injury_capacity_penalty":false,"juvenile_care_enabled":false,"ambush_predation_enabled":false,"local_coordination_enabled":false},"genetics":{"enabled":true,"mutation_per_mille":0,"extra_traits_enabled":false},"initial_state":initial_state}),
    )?;

    let mut generation = BTreeMap::<usize, usize>::new();
    for e in &world.entities {
        generation.insert(e.id, e.generation);
    }
    let mut bred = BTreeSet::<usize>::new();
    let mut births_species = BTreeMap::<String, usize>::new();
    let mut deaths_species = BTreeMap::<String, usize>::new();
    let mut births_gen = BTreeMap::<usize, usize>::new();
    let mut deaths_gen = BTreeMap::<usize, usize>::new();
    let mut death_causes = BTreeMap::<String, usize>::new();
    let mut lineage_issues = 0usize;
    let mut unknown_generation_births = 0usize;
    let mut entity_ticks = 0u64;
    let mut zero_energy_ticks = 0u64;
    let mut food_gained = 0u64;
    let mut movement_energy = 0u64;
    let mut first_zero_energy_tick: Option<usize> = None;
    let mut focal_first_zero_energy_tick: Option<usize> = None;
    let mut focal_death_tick: Option<usize> = None;
    let mut focal_death_cause: Option<String> = None;
    let mut blocked_approach_events = 0usize;
    let mut action_rejections = 0usize;
    let mut focal_zero_energy_visible_food_approach_failures = 0usize;
    let mut focal_underfunded_visible_food_approach_failures = 0usize;
    let mut reserve_intervention_count = 0usize;
    let mut samples = Vec::new();

    for tick in 1..=TICKS {
        let events = world.tick();
        if events.len() > MAX_EVENTS_PER_TICK {
            return Err(format!("event cap exceeded at tick {tick}").into());
        }
        if world.next_entity_id.saturating_sub(1) > MAX_IDENTITIES {
            return Err(format!("identity cap exceeded at tick {tick}").into());
        }
        let mut birth_lineage = Vec::new();
        let mut mating_events = Vec::new();
        let mut death_lineage = Vec::new();
        let mut tick_food = 0u64;
        let mut tick_move = 0u64;
        for event in &events {
            match event {
                SimulationEvent::EntityAte {
                    id,
                    pos,
                    amount,
                    corpse_id,
                } => {
                    if corpse_id.is_none() {
                        food_gained += *amount as u64;
                        tick_food += *amount as u64;
                    }
                    let _ = (id, pos);
                }
                SimulationEvent::EntityMoved { energy_cost, .. } => {
                    movement_energy += *energy_cost as u64;
                    tick_move += *energy_cost as u64;
                }
                SimulationEvent::ActionRejected {
                    id, action, reason, ..
                } => {
                    action_rejections += 1;
                    if action == "Approach"
                        && *reason == curious::simulation::event::RejectionReason::OutOfRange
                        && *id == 1
                    {
                        blocked_approach_events += 1;
                    }
                }
                SimulationEvent::EntityBorn { id, species, .. } => {
                    *births_species.entry(format!("{:?}", species)).or_default() += 1;
                    if let Some(e) = world.entities.iter().find(|e| e.id == *id) {
                        *births_gen.entry(e.generation).or_default() += 1;
                        generation.insert(*id, e.generation);
                        if e.parents.len() != 2 {
                            lineage_issues += 1;
                        }
                        let parents = e.parents.clone();
                        let parent_generations = parents
                            .iter()
                            .map(|p| generation.get(p).copied())
                            .collect::<Vec<_>>();
                        birth_lineage.push(json!({"id":id,"species":format!("{:?}",species),"generation":e.generation,"parent_ids":parents,"parent_generations":parent_generations}));
                    } else {
                        unknown_generation_births += 1;
                        lineage_issues += 1;
                        birth_lineage.push(json!({"id":id,"species":format!("{:?}",species),"generation":null,"parent_ids":null,"parent_generations":null}));
                    }
                }
                SimulationEvent::EntityDied {
                    id, species, cause, ..
                } => {
                    *deaths_species.entry(format!("{:?}", species)).or_default() += 1;
                    if let Some(g) = generation.get(id) {
                        *deaths_gen.entry(*g).or_default() += 1;
                    }
                    let cause = format!(
                        "{:?}",
                        cause.unwrap_or(curious::simulation::event::DeathCause::Unknown)
                    );
                    *death_causes.entry(cause.clone()).or_default() += 1;
                    if *id == 1 {
                        focal_death_tick = Some(tick);
                        focal_death_cause = Some(cause.clone());
                    }
                    death_lineage.push(json!({"id":id,"species":format!("{:?}",species),"generation":generation.get(id),"cause":cause}));
                }
                SimulationEvent::EntityMated {
                    parent1,
                    parent2,
                    child,
                    ..
                } => {
                    bred.insert(*parent1);
                    bred.insert(*parent2);
                    let child_slot = world.entities.iter().find(|e| e.id == *child);
                    mating_events.push(json!({"parent1":parent1,"parent1_generation":generation.get(parent1),"parent2":parent2,"parent2_generation":generation.get(parent2),"child":child,"child_generation":child_slot.map(|e|e.generation),"child_parent_ids":child_slot.map(|e|e.parents.clone())}));
                    if !generation.contains_key(parent1)
                        || !generation.contains_key(parent2)
                        || child_slot.is_none_or(|e| {
                            e.parents.len() != 2
                                || !e.parents.contains(parent1)
                                || !e.parents.contains(parent2)
                        })
                    {
                        lineage_issues += 1;
                    }
                }
                _ => {}
            }
        }
        let live = world
            .entities
            .iter()
            .filter(|e| alive(e))
            .collect::<Vec<_>>();
        entity_ticks += live.len() as u64;
        let zero = live.iter().filter(|e| e.base.life().energy == 0).count();
        zero_energy_ticks += zero as u64;
        if zero > 0 && first_zero_energy_tick.is_none() {
            first_zero_energy_tick = Some(tick);
        }
        let focal_predecision_zero = world
            .last_decisions
            .iter()
            .find(|d| d.id == 1)
            .is_some_and(|d| d.before.is_alive() && d.before.energy == 0);
        let focal_posttick_zero = world
            .entities
            .iter()
            .any(|e| e.id == 1 && alive(e) && e.base.life().energy == 0);
        if (focal_predecision_zero || focal_posttick_zero) && focal_first_zero_energy_tick.is_none()
        {
            focal_first_zero_energy_tick = Some(tick);
        }
        let mut reason_counts = BTreeMap::<String, usize>::new();
        let mut reserve_interventions = Vec::new();
        for decision in &world.last_decisions {
            let reason = format!("{:?}", decision.decision.reason);
            *reason_counts.entry(reason.clone()).or_default() += 1;
            if reason == "FoodSearchReserve" {
                reserve_intervention_count += 1;
                let life = &decision.before;
                let maintenance = curious::rules::biology::BMR_ENERGY_COST
                    .saturating_add(life.vision_metabolic_cost)
                    .saturating_add(life.stamina_metabolic_cost);
                let energy_cap = life.energy.saturating_sub(maintenance)
                    / life.move_energy_cost.saturating_mul(2).max(1);
                let remaining_speed_cap =
                    life.effective_speed(false).saturating_sub(life.moves_used);
                let applied_cap = energy_cap.min(remaining_speed_cap);
                let chosen_steps = match &decision.decision.intent {
                    curious::entity::intent::Intent::Move { steps } => steps.0.len(),
                    _ => 0,
                };
                reserve_interventions.push(json!({"id":decision.id,"decision_energy":life.energy,"movement_energy_cost":life.move_energy_cost,"known_next_tick_maintenance":maintenance,"energy_cap":energy_cap,"remaining_speed_cap":remaining_speed_cap,"applied_cap":applied_cap,"input_path_len":null,"chosen_steps":chosen_steps,"idle":chosen_steps==0,"moves_used":life.moves_used,"effective_speed":life.effective_speed(false),"intent":decision.decision.intent.clone(),"decision_reason":reason}));
            }
        }
        let (food_stock, water_stock) = resource_totals(&world);
        let focal_trace = world
            .last_decisions
            .iter()
            .find(|d| d.id == 1)
            .map(serde_json::to_value)
            .transpose()?;
        if let Some(decision) = world.last_decisions.iter().find(|d| d.id == 1) {
            let failed_approach = events.iter().any(|event| matches!(event, SimulationEvent::ActionRejected{id,action,reason,..} if *id == 1 && action == "Approach" && *reason == curious::simulation::event::RejectionReason::OutOfRange));
            if let curious::entity::intent::Intent::Eat {
                at,
                corpse_id: None,
            } = &decision.decision.intent
            {
                let matched_visible_plant_food = decision.perception.as_ref().is_some_and(|p| {
                    p.foods.iter().any(|f| {
                        !f.is_corpse && f.corpse_id.is_none() && f.amount > 0 && f.steps.0 == at.0
                    })
                });
                if failed_approach && matched_visible_plant_food {
                    let required =
                        at.0.len()
                            .saturating_sub(1)
                            .saturating_mul(decision.before.move_energy_cost);
                    let paid_move = events
                        .iter()
                        .filter_map(|event| match event {
                            SimulationEvent::EntityMoved {
                                id, energy_cost, ..
                            } if *id == 1 => Some(*energy_cost),
                            _ => None,
                        })
                        .sum::<usize>();
                    if required > 0 && decision.before.energy == 0 && paid_move == 0 {
                        focal_zero_energy_visible_food_approach_failures += 1;
                    }
                    if decision.before.energy < required && paid_move > 0 && paid_move < required {
                        focal_underfunded_visible_food_approach_failures += 1;
                    }
                }
            }
        }
        let focal_events = events
            .iter()
            .filter(|event| match event {
                SimulationEvent::EntityAte { id, .. }
                | SimulationEvent::EntityMoved { id, .. }
                | SimulationEvent::EntityDied { id, .. } => *id == 1,
                SimulationEvent::ActionRejected { id, .. } => *id == 1,
                _ => false,
            })
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()?;
        write_row(
            &mut writer,
            &mut output_size,
            &json!({"type":"tick","tick":tick,"alive":alive_counts(&world),"live_total":live.len(),"founders_alive":live.iter().filter(|e|founder_ids.contains(&e.id)).count(),"food_stock":food_stock,"water_stock":water_stock,"zero_energy_alive":zero,"plant_food_gained_energy":tick_food,"movement_energy_paid":tick_move,"plant_food_gained_cumulative":food_gained,"movement_energy_cumulative":movement_energy,"reason_counts":reason_counts,"reserve_interventions":reserve_interventions,"reserve_intervention_count_cumulative":reserve_intervention_count,"focal_first_zero_energy_tick":focal_first_zero_energy_tick,"focal_death_tick":focal_death_tick,"focal_decision_trace":focal_trace,"focal_events":focal_events,"focal_approach_out_of_range_count":blocked_approach_events,"focal_zero_energy_visible_food_approach_failures":focal_zero_energy_visible_food_approach_failures,"focal_underfunded_visible_food_approach_failures":focal_underfunded_visible_food_approach_failures,"action_rejections_cumulative":action_rejections,"birth_lineage":birth_lineage,"mating_events":mating_events,"death_lineage":death_lineage}),
        )?;
        if tick % 100 == 0 {
            samples.push(json!({"tick":tick,"living_entity_ticks":entity_ticks,"zero_energy_alive_tick_sum":zero_energy_ticks,"food_gained_energy":food_gained,"movement_energy_paid":movement_energy,"alive":alive_counts(&world),"food_stock":food_stock,"water_stock":water_stock,"f1_births":births_gen.get(&1).copied().unwrap_or(0),"f2_births":births_gen.get(&2).copied().unwrap_or(0)}));
        }
    }

    let living: BTreeSet<usize> = world
        .entities
        .iter()
        .filter(|e| alive(e))
        .map(|e| e.id)
        .collect();
    let f1_breeders: Vec<usize> = bred
        .iter()
        .filter(|id| generation.get(id) == Some(&1))
        .copied()
        .collect();
    let f2_breeders: Vec<usize> = bred
        .iter()
        .filter(|id| generation.get(id) == Some(&2))
        .copied()
        .collect();
    let living_f1_breeders: Vec<usize> = f1_breeders
        .iter()
        .filter(|id| living.contains(id))
        .copied()
        .collect();
    let living_f2_breeders: Vec<usize> = f2_breeders
        .iter()
        .filter(|id| living.contains(id))
        .copied()
        .collect();
    let living_descendant_breeders: Vec<usize> = bred
        .iter()
        .filter(|id| !founder_ids.contains(id) && living.contains(id))
        .copied()
        .collect();
    let checkpoint = State::capture(&world);
    let restored = checkpoint.clone().restore()?;
    let checkpoint_restore_valid = serde_json::to_value(State::capture(&restored))?
        == serde_json::to_value(checkpoint.clone())?;
    if !checkpoint_restore_valid {
        return Err("endpoint checkpoint restore changed State".into());
    }
    let final_state = serde_json::to_value(checkpoint)?;
    let summary = json!({"type":"summary","ticks_completed":TICKS,"living_entity_ticks":entity_ticks,"zero_energy_alive_tick_sum":zero_energy_ticks,"first_zero_energy_tick":first_zero_energy_tick,"focal_first_zero_energy_tick":focal_first_zero_energy_tick,"focal_death_tick":focal_death_tick,"focal_death_cause":focal_death_cause,"focal_ticks_from_first_zero_to_death":focal_first_zero_energy_tick.zip(focal_death_tick).map(|(zero,death)|death.saturating_sub(zero)),"plant_food_gained_energy":food_gained,"movement_energy_paid":movement_energy,"plant_food_to_movement_energy_ratio":if movement_energy>0 {json!(food_gained as f64/movement_energy as f64)} else {Value::Null},"focal_approach_out_of_range_count":blocked_approach_events,"focal_zero_energy_visible_food_approach_failures":focal_zero_energy_visible_food_approach_failures,"focal_underfunded_visible_food_approach_failures":focal_underfunded_visible_food_approach_failures,"reserve_intervention_count":reserve_intervention_count,"approach_rejection_energy_attribution":"separate explicit zero-energy/underfunded visible-plant-food counts; other OutOfRange events are unattributed","final_alive":alive_counts(&world),"founders_alive":living.intersection(&founder_ids).count(),"births_by_species":births_species,"deaths_by_species":deaths_species,"births_by_generation":births_gen,"deaths_by_generation":deaths_gen,"f1_births":births_gen.get(&1).copied().unwrap_or(0),"f2_births":births_gen.get(&2).copied().unwrap_or(0),"f1_breeders":f1_breeders,"living_f1_breeders":living_f1_breeders,"f2_breeders":f2_breeders,"living_f2_breeders":living_f2_breeders,"living_descendant_breeders":living_descendant_breeders,"death_causes":death_causes,"lineage_issues":lineage_issues,"unknown_generation_births":unknown_generation_births,"samples_every_100_ticks":samples,"checkpoint_restore_valid":checkpoint_restore_valid,"final_state":final_state});
    write_row(&mut writer, &mut output_size, &summary)?;
    println!("{{\"status\":\"complete\",\"ticks\":{TICKS},\"output_bytes\":{output_size}}}");
    Ok(())
}
fn arm_name(arm: Arm) -> &'static str {
    match arm {
        Arm::Baseline => "baseline",
        Arm::Candidate => "candidate",
    }
}
fn main() {
    if let Err(e) = run() {
        eprintln!("food_search_reserve_lab: {e}");
        std::process::exit(1);
    }
}
