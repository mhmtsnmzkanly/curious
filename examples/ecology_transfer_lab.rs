//! Bounded, per-feature ecology transfer screen. Runs one registered World.
//! See docs/ECOLOGY_TRANSFER_V1_PROTOCOL_20261026.md.
use curious::{
    entity::{genetics::Genome, phase::EntityPhase},
    environment::{SeasonCycle, SignalKind},
    scenario::Scenario,
    simulation::{SimulationEvent, checkpoint::State},
    world::World,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
};

const TICKS: usize = 600;
const MAX_IDENTITIES: usize = 10_000;
const MAX_EVENTS_PER_TICK: usize = 20_000;
const MAX_OUTPUT_BYTES: u64 = 1024 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Arm {
    Control,
    ClimateOnly,
    AmbushOnly,
    AlarmOnly,
    ExtraTraitsOnly,
}
impl Arm {
    fn parse(s: &str) -> Result<Self, String> {
        match s {
            "control" => Ok(Self::Control),
            "climate_only" => Ok(Self::ClimateOnly),
            "ambush_only" => Ok(Self::AmbushOnly),
            "alarm_only" => Ok(Self::AlarmOnly),
            "extra_traits_only" => Ok(Self::ExtraTraitsOnly),
            _ => Err(format!("unknown arm {s}")),
        }
    }
}

fn founder_genome(index: usize) -> Genome {
    let (vision, stamina) = match index % 4 {
        0 => (0, 0),
        1 => (1, 0),
        2 => (0, 1),
        _ => (1, 1),
    };
    Genome {
        mobility: 0,
        vision,
        stamina,
    }
}

fn configure_world(mut scenario: Scenario, seed: u64, arm: Arm) -> curious::Result<World> {
    scenario.genetics.enabled = true;
    scenario.genetics.mutation_per_mille = 0;
    scenario.genetics.extra_traits_enabled = arm == Arm::ExtraTraitsOnly;
    // Keep all unrelated policy options explicitly off in every arm.
    scenario.policy.mate_rendezvous = false;
    scenario.policy.reserve_opportunities = false;
    scenario.policy.water_residual_enabled = false;
    scenario.policy.injury_capacity_penalty = false;
    scenario.policy.juvenile_care_enabled = false;
    scenario.policy.ambush_predation_enabled = arm == Arm::AmbushOnly;
    scenario.policy.local_coordination_enabled = arm == Arm::AlarmOnly;
    for (index, founder) in scenario.founders.iter_mut().enumerate() {
        founder.genome = founder_genome(index);
    }
    let mut world = scenario.build(seed)?;
    if arm == Arm::ClimateOnly {
        world.environment.season_cycle = Some(SeasonCycle {
            period: 80,
            winter_bmr_surcharge: 2,
            winter_plant_regen_divisor: 2,
        });
    }
    Ok(world)
}

fn alive_counts(world: &World) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for e in &world.entities {
        if e.base.life().is_alive()
            && matches!(e.phase, EntityPhase::Active | EntityPhase::Sleeping { .. })
        {
            *out.entry(format!("{:?}", e.base.species())).or_default() += 1;
        }
    }
    out
}
fn resource_totals(world: &World) -> (usize, usize) {
    let mut food = 0;
    let mut water = 0;
    for y in world.map.min_y..=world.map.max_y {
        for x in world.map.min_x..=world.map.max_x {
            match world.map.cell(curious::map::movement::Position::new(x, y)) {
                Some(curious::map::cell::Cell::Food { amount }) => food += amount,
                Some(curious::map::cell::Cell::Water { amount }) => water += amount,
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
    if next > MAX_OUTPUT_BYTES {
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
            "usage: ecology_transfer_lab SCENARIO.json SEED ARM HABITAT OUTPUT.jsonl".into(),
        );
    }
    let seed: u64 = args[2].parse()?;
    let arm = Arm::parse(&args[3])?;
    let habitat = &args[4];
    let scenario_bytes = fs::read(&args[1])?;
    let scenario: Scenario = serde_json::from_slice(&scenario_bytes)?;
    let scenario_name = scenario.name.clone();
    let mut world = configure_world(scenario, seed, arm)?;
    let founder_ids: BTreeSet<usize> = world.entities.iter().map(|e| e.id).collect();
    let founder_genotypes: Vec<Value> = world.entities.iter().enumerate().map(|(i,e)| json!({"id":e.id,"species":format!("{:?}",e.base.species()),"genome":founder_genome(i)})).collect();
    let initial_state = serde_json::to_value(State::capture(&world))?;
    let path = Path::new(&args[5]);
    let file = OpenOptions::new().write(true).create_new(true).open(path)?;
    let mut writer = BufWriter::new(file);
    let mut output_size = 0;
    write_row(
        &mut writer,
        &mut output_size,
        &json!({"type":"header","schema":"curious-ecology-transfer-v1-jsonl","habitat":habitat,"scenario_name":scenario_name,"seed":seed,"arm":arm,"ticks":TICKS,"founders":founder_genotypes,"founder_ids":founder_ids,"genetics":{"enabled":true,"mutation_per_mille":0,"extra_traits_enabled":arm==Arm::ExtraTraitsOnly,"vision_stamina_alleles_transmitted_in_all_arms":true,"extra_trait_expression":arm==Arm::ExtraTraitsOnly,"extra_trait_rng_draws_per_mating":"0 in off arms; 2 in extra_traits_only even with mutation_per_mille=0"},"policy":{"mate_rendezvous":false,"reserve_opportunities":false,"water_residual_enabled":false,"injury_capacity_penalty":false,"juvenile_care_enabled":false,"ambush_predation_enabled":arm==Arm::AmbushOnly,"local_coordination_enabled":arm==Arm::AlarmOnly},"climate":if arm==Arm::ClimateOnly {json!({"period":80,"winter_bmr_surcharge":2,"winter_plant_regen_divisor":2})} else {Value::Null},"initial_state":initial_state}),
    )?;

    let mut species_births = BTreeMap::<String, usize>::new();
    let mut species_deaths = BTreeMap::<String, usize>::new();
    let mut births_by_generation = BTreeMap::<usize, usize>::new();
    let mut deaths_by_generation = BTreeMap::<usize, usize>::new();
    let mut death_causes = BTreeMap::<String, usize>::new();
    let mut generation = BTreeMap::<usize, usize>::new();
    for e in &world.entities {
        generation.insert(e.id, e.generation);
    }
    let mut bred = BTreeSet::<usize>::new();
    let mut lineage_issues = 0usize;
    let mut unknown_generation_births = 0usize;
    let mut attack_count = 0usize;
    let mut attack_damage = 0usize;
    let mut attack_kills = 0usize;
    let mut signal_emissions = 0usize;
    let mut observed_signal_receptions = 0usize;
    let mut decisions_with_perception = 0usize;
    let mut total_decision_traces = 0usize;
    let mut trait_event_count = 0usize;
    let mut event_requested_vision = 0usize;
    let mut event_requested_stamina = 0usize;
    let mut event_paid_vision = 0usize;
    let mut event_paid_stamina = 0usize;
    let mut entity_ticks = 0u64;
    let mut ticks_completed = 0usize;
    let mut samples = Vec::<Value>::new();
    let mut interval_births = BTreeMap::<String, usize>::new();
    let mut interval_deaths = BTreeMap::<String, usize>::new();
    let mut interval_entity_ticks = 0u64;
    let mut interval_winter_ticks = 0usize;
    let mut total_winter_ticks = 0usize;
    let mut interval_zero_energy = 0usize;
    let mut interval_low_health = 0usize;
    for tick in 1..=TICKS {
        let events = world.tick();
        ticks_completed = tick;
        let mut births_this_tick = BTreeMap::<String, usize>::new();
        let mut deaths_this_tick = BTreeMap::<String, usize>::new();
        let mut birth_lineage_this_tick = Vec::<Value>::new();
        let mut mating_events_this_tick = Vec::<Value>::new();
        let mut death_lineage_this_tick = Vec::<Value>::new();
        if events.len() > MAX_EVENTS_PER_TICK {
            return Err(format!("event cap exceeded at tick {tick}").into());
        }
        if world.next_entity_id.saturating_sub(1) > MAX_IDENTITIES {
            return Err(format!("identity cap exceeded at tick {tick}").into());
        }
        for event in &events {
            match event {
                SimulationEvent::EntityBorn { id, species, .. } => {
                    let key = format!("{:?}", species);
                    *species_births.entry(key.clone()).or_default() += 1;
                    *interval_births.entry(key.clone()).or_default() += 1;
                    *births_this_tick.entry(key).or_default() += 1;
                    if let Some(e) = world.entities.iter().find(|e| e.id == *id) {
                        *births_by_generation.entry(e.generation).or_default() += 1;
                        generation.insert(*id, e.generation);
                        if e.parents.len() != 2 {
                            lineage_issues += 1;
                        }
                        let parent_ids = e.parents.clone();
                        let parent_generations = parent_ids
                            .iter()
                            .map(|parent_id| generation.get(parent_id).copied())
                            .collect::<Vec<_>>();
                        birth_lineage_this_tick.push(json!({
                            "id":id,
                            "species":format!("{:?}",species),
                            "generation":e.generation,
                            "parent_ids":parent_ids,
                            "parent_generations":parent_generations,
                        }));
                    } else {
                        unknown_generation_births += 1;
                        lineage_issues += 1;
                        birth_lineage_this_tick.push(json!({
                            "id":id,
                            "species":format!("{:?}",species),
                            "generation":null,
                            "parent_ids":null,
                            "parent_generations":null,
                        }));
                    }
                }
                SimulationEvent::EntityDied {
                    id, species, cause, ..
                } => {
                    let key = format!("{:?}", species);
                    *species_deaths.entry(key.clone()).or_default() += 1;
                    *interval_deaths.entry(key.clone()).or_default() += 1;
                    *deaths_this_tick.entry(key).or_default() += 1;
                    if let Some(g) = generation.get(id) {
                        *deaths_by_generation.entry(*g).or_default() += 1;
                    }
                    *death_causes
                        .entry(format!(
                            "{:?}:{:?}",
                            species,
                            cause.unwrap_or(curious::simulation::event::DeathCause::Unknown)
                        ))
                        .or_default() += 1;
                    death_lineage_this_tick.push(json!({
                        "id":id,
                        "species":format!("{:?}",species),
                        "generation":generation.get(id).copied(),
                        "cause":format!("{:?}",cause.unwrap_or(curious::simulation::event::DeathCause::Unknown)),
                    }));
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
                    mating_events_this_tick.push(json!({
                        "parent1":parent1,
                        "parent1_generation":generation.get(parent1).copied(),
                        "parent2":parent2,
                        "parent2_generation":generation.get(parent2).copied(),
                        "child":child,
                        "child_generation":child_slot.map(|e|e.generation),
                        "child_parent_ids":child_slot.map(|e|e.parents.clone()),
                    }));
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
                SimulationEvent::EntityAttacked { damage, killed, .. } => {
                    attack_count += 1;
                    attack_damage += *damage;
                    attack_kills += usize::from(*killed);
                }
                SimulationEvent::EntityMetabolized {
                    vision_cost,
                    stamina_cost,
                    vision_paid,
                    stamina_paid,
                    ..
                } => {
                    trait_event_count += 1;
                    event_requested_vision += *vision_cost;
                    event_requested_stamina += *stamina_cost;
                    event_paid_vision += *vision_paid;
                    event_paid_stamina += *stamina_paid;
                }
                _ => {}
            }
        }
        let current_signals = world
            .environment
            .signals
            .iter()
            .filter(|s| s.kind == SignalKind::Sound && s.emitted_tick == tick)
            .count();
        signal_emissions += current_signals;
        total_decision_traces += world.last_decisions.len();
        for d in &world.last_decisions {
            if let Some(p) = &d.perception {
                decisions_with_perception += 1;
                observed_signal_receptions += p
                    .signals
                    .iter()
                    .filter(|s| s.kind == SignalKind::Sound)
                    .count();
            }
        }
        let counts = alive_counts(&world);
        let (food, water) = resource_totals(&world);
        let live_slots = world
            .entities
            .iter()
            .filter(|e| {
                e.base.life().is_alive()
                    && matches!(e.phase, EntityPhase::Active | EntityPhase::Sleeping { .. })
            })
            .collect::<Vec<_>>();
        entity_ticks += live_slots.len() as u64;
        interval_entity_ticks += live_slots.len() as u64;
        let zero_energy = live_slots
            .iter()
            .filter(|e| e.base.life().energy == 0)
            .count();
        let low_health = live_slots
            .iter()
            .filter(|e| e.base.life().health <= e.base.life().max_health / 4)
            .count();
        interval_zero_energy += zero_energy;
        interval_low_health += low_health;
        let winter = world
            .environment
            .season_cycle
            .is_some_and(|c| c.current_season(tick) == curious::environment::Season::Winter);
        interval_winter_ticks += usize::from(winter);
        total_winter_ticks += usize::from(winter);
        write_row(
            &mut writer,
            &mut output_size,
            &json!({"type":"tick","tick":tick,"alive":counts,"live_total":live_slots.len(),"founders_alive":live_slots.iter().filter(|e|founder_ids.contains(&e.id)).count(),"food_stock":food,"water_stock":water,"zero_energy_alive":zero_energy,"low_health_alive":low_health,"winter":winter,"births":births_this_tick,"deaths":deaths_this_tick,"birth_lineage":birth_lineage_this_tick,"mating_events":mating_events_this_tick,"death_lineage":death_lineage_this_tick}),
        )?;
        if tick % 100 == 0 {
            samples.push(json!({"tick":tick,"alive":counts,"founders_alive":live_slots.iter().filter(|e|founder_ids.contains(&e.id)).count(),"living_entity_ticks_interval":interval_entity_ticks,"living_entity_ticks_total":entity_ticks,"births_interval":interval_births,"deaths_interval":interval_deaths,"food_stock":food,"water_stock":water,"zero_energy_alive_tick_sum":interval_zero_energy,"low_health_alive_tick_sum":interval_low_health,"winter_ticks_interval":interval_winter_ticks}));
            interval_births.clear();
            interval_deaths.clear();
            interval_entity_ticks = 0;
            interval_zero_energy = 0;
            interval_low_health = 0;
            interval_winter_ticks = 0;
        }
    }
    let living: BTreeSet<usize> = world
        .entities
        .iter()
        .filter(|e| {
            e.base.life().is_alive()
                && matches!(e.phase, EntityPhase::Active | EntityPhase::Sleeping { .. })
        })
        .map(|e| e.id)
        .collect();
    let f1_breeders: Vec<usize> = bred
        .iter()
        .filter(|id| generation.get(id) == Some(&1))
        .copied()
        .collect();
    let living_offspring_breeders: Vec<usize> = bred
        .iter()
        .filter(|id| !founder_ids.contains(id) && living.contains(id))
        .copied()
        .collect();
    let living_f1_breeders: Vec<usize> = f1_breeders
        .iter()
        .filter(|id| living.contains(id))
        .copied()
        .collect();
    let f2_births = births_by_generation.get(&2).copied().unwrap_or(0);
    let trait_status = if arm == Arm::ExtraTraitsOnly {
        json!({"requested_body_costs_by_living_id":world.entities.iter().filter(|e|living.contains(&e.id)).map(|e|json!({"id":e.id,"vision_requested_per_tick":e.base.life().vision_metabolic_cost,"stamina_requested_per_tick":e.base.life().stamina_metabolic_cost})).collect::<Vec<_>>(),"entity_metabolized_event_count":trait_event_count,"event_reported_vision_requested":event_requested_vision,"event_reported_stamina_requested":event_requested_stamina,"event_recorded_vision_paid":event_paid_vision,"event_recorded_stamina_paid":event_paid_stamina,"actual_total_paid_outside_event_coverage":"unknown"})
    } else {
        Value::Null
    };
    let final_checkpoint = State::capture(&world);
    let restored = final_checkpoint.clone().restore()?;
    let checkpoint_restore_valid = serde_json::to_value(State::capture(&restored))?
        == serde_json::to_value(final_checkpoint.clone())?;
    if !checkpoint_restore_valid {
        return Err("final checkpoint restore changed State".into());
    }
    let final_state = serde_json::to_value(final_checkpoint)?;
    let summary = json!({"type":"summary","ticks_completed":ticks_completed,"living_entity_ticks":entity_ticks,"final_alive":alive_counts(&world),"founders_alive":living.intersection(&founder_ids).count(),"births_by_species":species_births,"deaths_by_species":species_deaths,"births_by_generation":births_by_generation,"deaths_by_generation":deaths_by_generation,"unknown_generation_births":unknown_generation_births,"lineage_issues":lineage_issues,"f1_births":births_by_generation.get(&1).copied().unwrap_or(0),"f2_births":f2_births,"f1_breeders":f1_breeders,"living_f1_breeders":living_f1_breeders,"living_offspring_breeders":living_offspring_breeders,"death_causes":death_causes,"attacks":{"count":attack_count,"damage":attack_damage,"kills":attack_kills},"alarm":{"emitted_sound_signals":signal_emissions,"observed_sound_receptions":if decisions_with_perception>0 && decisions_with_perception==total_decision_traces {json!(observed_signal_receptions)} else {Value::Null},"decisions_with_perception":decisions_with_perception,"decision_traces":total_decision_traces,"reception_observation_coverage":"complete only when every captured decision trace includes Perception"},"climate":{"seasonal_world":arm==Arm::ClimateOnly,"cycle_ticks":world.tick_counter,"winter_ticks":total_winter_ticks,"winter_stress_body_counts_in_samples":samples.iter().map(|s|json!({"tick":s["tick"],"zero_energy_alive_tick_sum":s["zero_energy_alive_tick_sum"],"low_health_alive_tick_sum":s["low_health_alive_tick_sum"]})).collect::<Vec<_>>()},"extra_traits":trait_status,"samples_every_100_ticks":samples,"checkpoint_restore_valid":checkpoint_restore_valid,"final_state":final_state});
    write_row(&mut writer, &mut output_size, &summary)?;
    println!(
        "{{\"status\":\"complete\",\"ticks\":{ticks_completed},\"output_bytes\":{output_size}}}"
    );
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("ecology_transfer_lab: {e}");
        std::process::exit(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arms_are_isolated() {
        let control = configure_world(
            serde_json::from_str(include_str!("../scenarios/meadow.json")).unwrap(),
            7,
            Arm::Control,
        )
        .unwrap();
        for arm in [
            Arm::Control,
            Arm::ClimateOnly,
            Arm::AmbushOnly,
            Arm::AlarmOnly,
            Arm::ExtraTraitsOnly,
        ] {
            let w = configure_world(
                serde_json::from_str(include_str!("../scenarios/meadow.json")).unwrap(),
                7,
                arm,
            )
            .unwrap();
            assert_eq!(w.rng.state(), control.rng.state());
            assert_eq!(w.entities.len(), control.entities.len());
            for (entity, baseline) in w.entities.iter().zip(&control.entities) {
                assert_eq!(entity.pos, baseline.pos);
                assert_eq!(entity.genome, baseline.genome);
                if arm != Arm::ExtraTraitsOnly {
                    assert_eq!(
                        entity.base.life().vision_range,
                        baseline.base.life().vision_range
                    );
                    assert_eq!(
                        entity.base.life().max_stamina,
                        baseline.base.life().max_stamina
                    );
                    assert_eq!(
                        entity.base.life().vision_metabolic_cost,
                        baseline.base.life().vision_metabolic_cost
                    );
                    assert_eq!(
                        entity.base.life().stamina_metabolic_cost,
                        baseline.base.life().stamina_metabolic_cost
                    );
                }
            }
            assert!(w.genetics.enabled);
            assert_eq!(w.genetics.mutation_per_mille, 0);
            assert_eq!(w.genetics.extra_traits_enabled, arm == Arm::ExtraTraitsOnly);
            assert_eq!(w.policy.ambush_predation_enabled, arm == Arm::AmbushOnly);
            assert_eq!(w.policy.local_coordination_enabled, arm == Arm::AlarmOnly);
            assert!(!w.policy.juvenile_care_enabled);
            assert!(!w.policy.mate_rendezvous);
            assert!(!w.policy.reserve_opportunities);
            assert_eq!(
                w.environment.season_cycle.is_some(),
                arm == Arm::ClimateOnly
            );
            assert!(w.entities.iter().all(|e| e.genome.mobility == 0));
            if arm == Arm::ExtraTraitsOnly {
                assert!(
                    w.entities
                        .iter()
                        .any(|e| e.base.life().vision_metabolic_cost > 0
                            || e.base.life().stamina_metabolic_cost > 0)
                );
            }
        }
    }
    #[test]
    fn founder_genotype_templates_repeat_without_mobility() {
        let got = (0..8).map(|i| founder_genome(i)).collect::<Vec<_>>();
        assert_eq!(&got[..4], &got[4..]);
        assert_eq!(
            got.iter()
                .map(|g| (g.vision, g.stamina))
                .collect::<Vec<_>>(),
            vec![
                (0, 0),
                (1, 0),
                (0, 1),
                (1, 1),
                (0, 0),
                (1, 0),
                (0, 1),
                (1, 1)
            ]
        );
        assert!(got.iter().all(|g| g.mobility == 0));
    }
}
