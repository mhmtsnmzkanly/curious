//! Paired bounded vision-selection capture. The audit is observational only.
#[path = "support/ecology_audit.rs"]
#[allow(dead_code)]
mod ecology_audit;
#[path = "support/food_redistribution.rs"]
mod food_redistribution;
#[path = "support/genotype_audit.rs"]
#[allow(dead_code)]
mod genotype_audit;

use curious::{
    creatures::{carnivore::CarnivoreEntity, herbivore::HerbivoreEntity},
    entity::{
        decision::DecisionTrace, genetics::Genome, perception::Perception, phase::EntityPhase,
        species::Species,
    },
    scenario::{Founder, Scenario},
    simulation::{
        SimulationEvent,
        checkpoint::{Individual, State},
    },
    world::World,
};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
};

const ENGINEERING_TICKS: usize = 200;
const FORMAL_TICKS: usize = 500;
const ENGINEERING_SEED: u64 = 123;
const FORMAL_SEEDS: [u64; 4] = [55001, 55002, 55003, 55004];
const MAX_RAW_BYTES: u64 = 240 * 1024 * 1024;
const MAX_IDENTITIES: usize = 50_000;
const MAX_EVENTS_PER_TICK: usize = 20_000;
const MAX_LINEAGE_EVENTS: usize = 50_000;
const MAX_DETAIL_ROWS: usize = 2_000;
const MAX_FRAME_PATH_SAMPLE_BYTES: u64 = 16 * 1024 * 1024;
const PRIOR_RAW_BYTES: u64 = 80_367_647;
const PRIOR_TICKS: u64 = 400;
const PRIOR_TRACE_ROWS: u64 = 258_470;
#[derive(Clone, Debug, Serialize)]
struct FounderRow {
    id: usize,
    species: Species,
    x: isize,
    y: isize,
    tile_row: usize,
    tile_column: usize,
    genome: Genome,
    base_vision_range: usize,
}

#[derive(Clone, Debug, Serialize)]
struct TileCount {
    row: usize,
    column: usize,
    vision_0: usize,
    vision_1: usize,
}

#[derive(Clone, Debug, Serialize)]
struct AssignmentReport {
    founders: Vec<FounderRow>,
    tile_counts: Vec<TileCount>,
    herbivores: usize,
    vision_0: usize,
    vision_1: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    AcceptedNeutral,
    AcceptedExpressed,
    PatchedNeutral,
    PatchedExpressed,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Self::AcceptedNeutral => "accepted_neutral",
            Self::AcceptedExpressed => "accepted_expressed",
            Self::PatchedNeutral => "patched_neutral",
            Self::PatchedExpressed => "patched_expressed",
        }
    }
    fn extra_traits(self) -> bool {
        matches!(self, Self::AcceptedExpressed | Self::PatchedExpressed)
    }
    fn patched(self) -> bool {
        matches!(self, Self::PatchedNeutral | Self::PatchedExpressed)
    }
}

fn tile_coordinate(value: isize, min: isize, max: isize) -> Result<usize, String> {
    if min > max || value < min || value > max {
        return Err("founder position is outside scenario bounds".into());
    }
    let span = (max - min + 1) as i128;
    let offset = (value - min) as i128;
    Ok(((offset * 4) / span).min(3) as usize)
}

fn base_life(founder: &Founder) -> curious::entity::lifestate::LifeState {
    founder
        .life
        .clone()
        .unwrap_or_else(|| match &founder.species {
            Species::Herbivore => HerbivoreEntity::default().life_state,
            Species::Carnivore => CarnivoreEntity::default().life_state,
            Species::Omnivore => curious::creatures::omnivore::OmnivoreEntity::default().life_state,
        })
}

/// Deterministic, 4x4 spatially stratified, globally alternating assignment.
/// This helper is pure and never constructs a World.
fn assign_founders(scenario: &mut Scenario) -> Result<AssignmentReport, String> {
    let [min_x, max_x, min_y, max_y] = scenario.bounds;
    if scenario.bounds.iter().any(|v| v.unsigned_abs() > 1_000_000)
        || min_x > max_x
        || min_y > max_y
    {
        return Err("invalid scenario bounds".into());
    }
    let mut tile_members: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
    let mut tiles = vec![[0usize; 2]; 16];
    let mut rows = Vec::with_capacity(scenario.founders.len());
    for (index, founder) in scenario.founders.iter().enumerate() {
        let row = tile_coordinate(founder.pos.y, min_y, max_y)?;
        let column = tile_coordinate(founder.pos.x, min_x, max_x)?;
        rows.push(FounderRow {
            id: index + 1,
            species: founder.species.clone(),
            x: founder.pos.x,
            y: founder.pos.y,
            tile_row: row,
            tile_column: column,
            genome: Genome::default(),
            base_vision_range: base_life(founder).vision_range,
        });
        if founder.species == Species::Herbivore {
            tile_members.entry((row, column)).or_default().push(index);
        }
    }
    let herbivore_count: usize = tile_members.values().map(Vec::len).sum();
    if herbivore_count == 0 || herbivore_count % 2 != 0 {
        return Err("herbivore founder count must be positive and even".into());
    }
    let mut alternating = 0usize;
    for ((row, column), indices) in &mut tile_members {
        indices.sort_by_key(|&i| {
            let f = &scenario.founders[i];
            (f.pos.y, f.pos.x, i)
        });
        for &index in indices.iter() {
            let allele = alternating % 2;
            alternating += 1;
            tiles[row * 4 + column][allele] += 1;
            rows[index].genome = Genome {
                mobility: 0,
                vision: allele as u8,
                stamina: 0,
            };
        }
    }
    for (index, founder) in scenario.founders.iter_mut().enumerate() {
        if founder.species != Species::Herbivore {
            rows[index].genome = Genome::default();
        }
        founder.genome = rows[index].genome.clone();
    }
    let vision_0 = rows
        .iter()
        .filter(|r| r.species == Species::Herbivore && r.genome.vision == 0)
        .count();
    let vision_1 = rows
        .iter()
        .filter(|r| r.species == Species::Herbivore && r.genome.vision == 1)
        .count();
    if vision_0 != herbivore_count / 2 || vision_1 != herbivore_count / 2 {
        return Err("spatial assignment did not yield exact global 1:1 split".into());
    }
    let tile_counts = tiles
        .into_iter()
        .enumerate()
        .map(|(i, values)| TileCount {
            row: i / 4,
            column: i % 4,
            vision_0: values[0],
            vision_1: values[1],
        })
        .collect();
    Ok(AssignmentReport {
        founders: rows,
        tile_counts,
        herbivores: herbivore_count,
        vision_0,
        vision_1,
    })
}

fn explicit_profile(scenario: &mut Scenario, arm: Arm) {
    scenario.genetics.enabled = true;
    scenario.genetics.mutation_per_mille = 0;
    scenario.genetics.extra_traits_enabled = arm.extra_traits();
    scenario.policy.mate_rendezvous = true;
    scenario.policy.reserve_opportunities = true;
    scenario.policy.water_residual_enabled = false;
}

fn configure_agents(world: &mut World) {
    world.observation.full_all = true;
    world.observation.full_for.clear();
    for entity in &mut world.entities {
        entity.learner.enabled = true;
        entity.learner.update_enabled = true;
        entity.memory.enabled = true;
        entity.memory.companions_enabled = true;
        entity.hunt_learner.enabled = false;
        entity.memory.refuge_outcomes_enabled = false;
        entity.memory.refuge_generalization_enabled = false;
        entity.memory.refuge_generalization_update_enabled = false;
    }
}

fn verify_initial(world: &World, arm: Arm, assignment: &AssignmentReport) -> Result<(), String> {
    if world.entities.len() != assignment.founders.len() {
        return Err("world founder count differs from scenario assignment".into());
    }
    if !world.genetics.enabled
        || world.genetics.mutation_per_mille != 0
        || world.genetics.extra_traits_enabled != arm.extra_traits()
        || !world.policy.mate_rendezvous
        || !world.policy.reserve_opportunities
        || world.policy.water_residual_enabled
    {
        return Err("actual genetics or accepted policy profile mismatch".into());
    }
    for row in &assignment.founders {
        let Some(slot) = world.entities.iter().find(|slot| slot.id == row.id) else {
            return Err(format!("initial State missing founder {}", row.id));
        };
        let life = slot.base.life();
        let expected_vision = if arm.extra_traits() && row.species == Species::Herbivore {
            (row.base_vision_range as isize + row.genome.vision as isize).clamp(1, 32) as usize
        } else {
            row.base_vision_range
        };
        let expected_cost = if arm.extra_traits() && row.species == Species::Herbivore {
            row.genome.vision as usize
        } else {
            0
        };
        if slot.genome != row.genome
            || life.vision_range != expected_vision
            || life.vision_metabolic_cost != expected_cost
            || life.stamina_metabolic_cost != 0
        {
            return Err(format!(
                "founder {} genotype/phenotype expression mismatch",
                row.id
            ));
        }
    }
    Ok(())
}

fn audit_relevant(event: &SimulationEvent) -> bool {
    matches!(
        event,
        SimulationEvent::ResourceReplenished { .. }
            | SimulationEvent::EntityAte { .. }
            | SimulationEvent::EntityDrank { .. }
            | SimulationEvent::EntityMetabolized { .. }
            | SimulationEvent::EntityMated { .. }
            | SimulationEvent::EntityBorn { .. }
            | SimulationEvent::EntityDied { .. }
    )
}

fn relevant_events(
    events: &[SimulationEvent],
    after: &State,
) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let after_by: BTreeMap<_, _> = after.entities.iter().map(|i| (i.id, i)).collect();
    let mut rows = Vec::new();
    for event in events.iter().filter(|e| audit_relevant(e)) {
        let value = serde_json::to_value(event)?;
        let child = match event {
            SimulationEvent::EntityBorn { id, .. } => after_by.get(id).map(|i| json!({
                "id":i.id,"species":i.species,"genome":i.genome,"generation":i.generation,"parents":i.parents,
                "vision_range":i.life.vision_range,"vision_cost_requested":i.life.vision_metabolic_cost,
                "stamina":i.life.stamina,"max_stamina":i.life.max_stamina,"stamina_cost_requested":i.life.stamina_metabolic_cost
            })),
            _ => None,
        };
        rows.push(match child {
            Some(snapshot) => json!([value, snapshot]),
            None => value,
        });
    }
    Ok(rows)
}

fn compact_traces(
    traces: &[DecisionTrace],
    before: &State,
) -> (Vec<Value>, Vec<usize>, Vec<usize>) {
    let expected: Vec<_> = before.entities.iter().map(|i| i.id).collect();
    let missing = missing_trace_ids(&expected, traces);
    let mut perception_missing = Vec::new();
    let rows = traces
        .iter()
        .map(|trace| {
            let counts = food_trace_counts(trace.perception.as_ref());
            if counts.is_none() {
                perception_missing.push(trace.id);
            }
            let (plant_histogram, corpse_histogram) = match counts {
                Some((plant, corpse)) => (Some(plant), Some(corpse)),
                None => (None, None),
            };
            json!([
                trace.id,
                trace.perception.is_some(),
                trace.before.vision_range,
                plant_histogram,
                corpse_histogram
            ])
        })
        .collect();
    (rows, missing, perception_missing)
}

fn missing_trace_ids(expected_ids: &[usize], traces: &[DecisionTrace]) -> Vec<usize> {
    let seen: BTreeSet<_> = traces.iter().map(|trace| trace.id).collect();
    expected_ids
        .iter()
        .copied()
        .filter(|id| !seen.contains(id))
        .collect()
}

fn food_trace_counts(
    perception: Option<&Perception>,
) -> Option<(Vec<[usize; 2]>, Vec<[usize; 2]>)> {
    let perception = perception?;
    let mut plant_paths_by_length = BTreeMap::new();
    let mut corpse_paths_by_length = BTreeMap::new();
    for food in &perception.foods {
        let length = food.steps.len();
        let histogram = if food.is_corpse {
            &mut corpse_paths_by_length
        } else {
            &mut plant_paths_by_length
        };
        *histogram.entry(length).or_default() += 1;
    }
    Some((
        plant_paths_by_length
            .into_iter()
            .map(|(length, count)| [length, count])
            .collect(),
        corpse_paths_by_length
            .into_iter()
            .map(|(length, count)| [length, count])
            .collect(),
    ))
}

fn path_histogram_metrics(histogram: &[[usize; 2]]) -> (usize, usize, bool) {
    let mut at_or_below = 0usize;
    let mut beyond = 0usize;
    for [length, count] in histogram {
        if *length <= 6 {
            at_or_below += count;
        } else {
            beyond += count;
        }
    }
    (at_or_below, beyond, at_or_below > 0)
}

fn full_food_path_samples(traces: &[DecisionTrace]) -> Vec<Value> {
    traces
        .iter()
        .filter(|trace| trace.species == Species::Herbivore)
        .map(|trace| {
            let paths = trace.perception.as_ref().map(|perception| {
                perception
                    .foods
                    .iter()
                    .map(|food| json!({"is_corpse":food.is_corpse,"steps":food.steps.0}))
                    .collect::<Vec<_>>()
            });
            json!({"id":trace.id,"perception_known":paths.is_some(),"food_paths":paths})
        })
        .collect()
}

fn capture_size_forecast(compact_trace_row_bytes: u64) -> Option<u64> {
    let planned_ticks = 4u64 * ENGINEERING_TICKS as u64;
    let tick_scale = planned_ticks.checked_div(PRIOR_TICKS)?;
    let baseline = PRIOR_RAW_BYTES.checked_mul(tick_scale)?;
    let trace_rows = PRIOR_TRACE_ROWS.checked_mul(tick_scale)?;
    let compact_rows = trace_rows.checked_mul(compact_trace_row_bytes)?;
    let with_frame_samples = baseline
        .checked_add(compact_rows)?
        .checked_add(MAX_FRAME_PATH_SAMPLE_BYTES)?;
    // Reserve 10% over this deliberately conservative estimate. The baseline
    // already contains older compact trace rows, so adding the entire new row
    // payload overcounts the replacement instead of subtracting uncertain old
    // bytes.
    with_frame_samples.checked_mul(1100)?.checked_div(1000)
}

fn compact_trace_row_forecast_bytes() -> u64 {
    serde_json::to_vec(&json!([
        123456,
        true,
        7,
        [[1, 12], [4, 3], [6, 2], [7, 1]],
        [[2, 2], [6, 1], [7, 1]]
    ]))
    .map(|row| row.len() as u64)
    .unwrap_or(u64::MAX)
}

fn trait_state_mismatches(state: &State) -> Vec<Value> {
    state.entities.iter().filter_map(|i| {
        let expected_vision=if state.genetics.extra_traits_enabled {i.genome.vision as usize}else{0};
        let expected_stamina=if state.genetics.extra_traits_enabled {i.genome.stamina as usize}else{0};
        (i.life.vision_metabolic_cost!=expected_vision || i.life.stamina_metabolic_cost!=expected_stamina).then(||json!({
            "id":i.id,"species":i.species,"genome":i.genome,"life_vision_cost":i.life.vision_metabolic_cost,"expected_vision_cost":expected_vision,
            "life_stamina_cost":i.life.stamina_metabolic_cost,"expected_stamina_cost":expected_stamina
        }))
    }).collect()
}

fn trait_event_mismatches(events: &[SimulationEvent], before: &State, after: &State) -> Vec<Value> {
    let before_by: BTreeMap<_, _> = before.entities.iter().map(|i| (i.id, i)).collect();
    let after_by: BTreeMap<_, _> = after.entities.iter().map(|i| (i.id, i)).collect();
    events.iter().filter_map(|event| if let SimulationEvent::EntityMetabolized{id,vision_cost,stamina_cost,vision_paid,stamina_paid} = event {
        let actor=before_by.get(id).or_else(||after_by.get(id));
        let expected=actor.map(|i|(i.life.vision_metabolic_cost,i.life.stamina_metabolic_cost));
        let mismatch=expected.is_none_or(|(v,s)|v!=*vision_cost||s!=*stamina_cost)||vision_paid>vision_cost||stamina_paid>stamina_cost;
        mismatch.then(||json!({"id":id,"vision_requested":vision_cost,"stamina_requested":stamina_cost,"vision_paid":vision_paid,"stamina_paid":stamina_paid,"expected_life_cost":expected}))
    } else {None}).collect()
}

fn live_counts(world: &World) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for slot in &world.entities {
        if slot.base.life().is_alive()
            && matches!(
                slot.phase,
                EntityPhase::Active | EntityPhase::Sleeping { .. }
            )
        {
            *counts
                .entry(format!("{:?}", slot.base.species()))
                .or_default() += 1;
        }
    }
    counts
}

fn genotype_key(species: &Species, genome: &Genome) -> String {
    format!(
        "{:?}:m{}v{}s{}",
        species, genome.mobility, genome.vision, genome.stamina
    )
}

fn exposed_alive(individual: &Individual) -> bool {
    individual.life.is_alive()
        && !matches!(
            individual.phase,
            EntityPhase::Corpse { .. } | EntityPhase::Removed
        )
}

fn living_genotype_counts(state: &State) -> BTreeMap<String, u64> {
    let mut counts = BTreeMap::new();
    for individual in state.entities.iter().filter(|i| exposed_alive(i)) {
        *counts
            .entry(genotype_key(&individual.species, &individual.genome))
            .or_insert(0) += 1;
    }
    counts
}

fn ancestry_mask_union(
    parent1: usize,
    parent2: usize,
    child: usize,
    ancestry: &mut BTreeMap<usize, Vec<u64>>,
    words: usize,
) -> Result<(), String> {
    let left = ancestry
        .get(&parent1)
        .ok_or_else(|| format!("missing founder ancestry for parent {parent1}"))?;
    let right = ancestry
        .get(&parent2)
        .ok_or_else(|| format!("missing founder ancestry for parent {parent2}"))?;
    if left.len() != words || right.len() != words {
        return Err("founder ancestry bitset length mismatch".into());
    }
    let merged = left.iter().zip(right).map(|(a, b)| a | b).collect();
    ancestry.insert(child, merged);
    Ok(())
}

fn primary_and_family_summary(
    audit: &genotype_audit::Audit,
    ids: &BTreeSet<usize>,
    ancestry: &BTreeMap<usize, Vec<u64>>,
    founder_ids: &[usize],
) -> Result<Value, String> {
    let mut founders = [0u64; 2];
    let mut breeders = [0u64; 2];
    let mut parent_pairs: [BTreeSet<(usize, usize)>; 2] = [BTreeSet::new(), BTreeSet::new()];
    let mut ancestor_union: [Vec<u64>; 2] =
        std::array::from_fn(|_| vec![0u64; founder_ids.len().div_ceil(64)]);
    for id in ids {
        let Some(individual) = audit.individual(*id) else {
            continue;
        };
        if individual.species != Species::Herbivore {
            continue;
        }
        if individual.genome.vision > 1 {
            return Err(format!(
                "herbivore vision allele outside frozen 0/1 study at id {id}"
            ));
        }
        let allele = individual.genome.vision as usize;
        if individual.founder {
            founders[allele] += 1;
        }
        if individual.born && individual.lineage_known && individual.alive && individual.mated {
            breeders[allele] += 1;
            if individual.parents.len() == 2 {
                let mut pair = [individual.parents[0], individual.parents[1]];
                pair.sort_unstable();
                parent_pairs[allele].insert((pair[0], pair[1]));
            }
            if let Some(mask) = ancestry.get(id) {
                for (target, source) in ancestor_union[allele].iter_mut().zip(mask) {
                    *target |= *source;
                }
            }
        }
    }
    let founder_total = founders[0] + founders[1];
    let breeder_total = breeders[0] + breeders[1];
    let founder_p = allele_frequency(founders[1], founder_total);
    let breeder_p = allele_frequency(breeders[1], breeder_total);
    let delta = within_arm_change(founder_p, breeder_p);
    let family_pairs = parent_pairs
        .iter()
        .map(|families| families.iter().map(|(a, b)| [*a, *b]).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let ancestor_ids = ancestor_union
        .iter()
        .map(|mask| {
            founder_ids
                .iter()
                .enumerate()
                .filter_map(|(i, id)| ((mask[i / 64] >> (i % 64)) & 1 == 1).then_some(*id))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "founder_vision_allele_counts":{"0":founders[0],"1":founders[1]},
        "founder_vision_1_frequency":founder_p,
        "living_descendant_breeder_counts":{"0":breeders[0],"1":breeders[1],"total":breeder_total},
        "living_descendant_breeder_vision_1_frequency":breeder_p,
        "within_arm_founder_to_breeder_change":delta,
        "living_descendant_breeder_parent_pair_family_counts":{"0":parent_pairs[0].len(),"1":parent_pairs[1].len()},
        "living_descendant_breeder_parent_pair_family_ids_by_allele":family_pairs,
        "living_descendant_breeder_founder_ancestor_union_counts":{"0":ancestor_ids[0].len(),"1":ancestor_ids[1].len()},
        "living_descendant_breeder_founder_ancestor_ids_by_allele":ancestor_ids,
        "family_counts_are_additional_replicates":false
    }))
}

fn allele_frequency(allele_one: u64, denominator: u64) -> Option<f64> {
    (denominator > 0).then(|| allele_one as f64 / denominator as f64)
}

fn within_arm_change(
    founder_frequency: Option<f64>,
    breeder_frequency: Option<f64>,
) -> Option<f64> {
    Some(breeder_frequency? - founder_frequency?)
}

fn paired_change_delta(
    expressed_founder: Option<f64>,
    expressed_breeder: Option<f64>,
    neutral_founder: Option<f64>,
    neutral_breeder: Option<f64>,
) -> Option<f64> {
    Some(
        within_arm_change(expressed_founder, expressed_breeder)?
            - within_arm_change(neutral_founder, neutral_breeder)?,
    )
}

fn add_exposure_rates(summary: &mut Value, ticks: &BTreeMap<String, u64>) {
    let Some(rows) = summary.get_mut("genotypes").and_then(Value::as_object_mut) else {
        return;
    };
    for (key, row) in rows {
        let denominator = ticks.get(key).copied().unwrap_or(0);
        let Some(values) = row.as_object_mut() else {
            continue;
        };
        values.insert(
            "living_genotype_individual_ticks".into(),
            json!(denominator),
        );
        let numerators = [
            (
                "vision_requested",
                "vision_requested_per_living_individual_tick",
            ),
            ("vision_paid", "vision_paid_per_living_individual_tick"),
            ("vision_unpaid", "vision_unpaid_per_living_individual_tick"),
            (
                "stamina_requested",
                "stamina_requested_per_living_individual_tick",
            ),
            ("stamina_paid", "stamina_paid_per_living_individual_tick"),
            (
                "stamina_unpaid",
                "stamina_unpaid_per_living_individual_tick",
            ),
            ("plant_food", "plant_food_per_living_individual_tick"),
            ("corpse_food", "corpse_food_per_living_individual_tick"),
            ("water", "water_per_living_individual_tick"),
        ];
        for (source, target) in numerators {
            let numerator = values.get(source).and_then(Value::as_u64).unwrap_or(0);
            let rate = (denominator > 0).then(|| numerator as f64 / denominator as f64);
            values.insert(target.into(), json!(rate));
        }
    }
}

fn stock_reconciliation(audit: &ecology_audit::Audit) -> Option<Value> {
    audit.reconciliation.last().cloned()
}

fn projected_ecology_issues(
    audit: &ecology_audit::Audit,
    world: &World,
    events: &[SimulationEvent],
) -> usize {
    let mut parented = audit.parented_births.clone();
    let mut count = 0usize;
    for event in events {
        match event {
            SimulationEvent::EntityMated {
                parent1,
                parent2,
                child,
                ..
            } => {
                if !audit.ledger.contains_key(parent1) || !audit.ledger.contains_key(parent2) {
                    count += 1;
                }
                let expected = audit
                    .ledger
                    .get(parent1)
                    .and_then(|x| x.generation)
                    .zip(audit.ledger.get(parent2).and_then(|x| x.generation))
                    .map(|(a, b)| a.max(b) + 1);
                if let Some(c) = world.entities.iter().find(|x| x.id == *child) {
                    if expected.is_some_and(|g| g != c.generation)
                        || c.parents.len() != 2
                        || !c.parents.contains(parent1)
                        || !c.parents.contains(parent2)
                    {
                        count += 1;
                    }
                } else {
                    count += 1;
                }
                parented.insert(*child);
            }
            SimulationEvent::EntityBorn { id, species, .. } => {
                if audit.ledger.get(id).is_some_and(|x| x.species != *species)
                    || !parented.contains(id)
                {
                    count += 1;
                }
            }
            SimulationEvent::EntityAte { id, .. }
            | SimulationEvent::EntityDrank { id, .. }
            | SimulationEvent::EntityMoved { id, .. } => {
                if !audit.ledger.contains_key(id) {
                    count += 1;
                }
            }
            SimulationEvent::EntityMetabolized {
                id,
                vision_cost,
                stamina_cost,
                vision_paid,
                stamina_paid,
            } => {
                if vision_paid > vision_cost || stamina_paid > stamina_cost {
                    count += 1;
                }
                if !audit.ledger.contains_key(id) {
                    count += 1;
                }
            }
            _ => {}
        }
    }
    count
}

fn write_line(
    writer: &mut BufWriter<std::fs::File>,
    bytes: &mut u64,
    value: &Value,
    cap: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let row = serde_json::to_vec(value)?;
    let next = bytes.saturating_add(row.len() as u64).saturating_add(1);
    if next > cap {
        return Err("paired-seed raw JSONL cap exceeded".into());
    }
    writer.write_all(&row)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    *bytes = next;
    Ok(())
}

fn run_arm(
    scenario_bytes: &[u8],
    arm: Arm,
    seed: u64,
    ticks: usize,
    frame_ticks: &[usize],
    raw_cap: u64,
    frame_sample_bytes: &mut u64,
    raw: &mut BufWriter<std::fs::File>,
    raw_bytes: &mut u64,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    let mut scenario: Scenario = serde_json::from_slice(scenario_bytes)?;
    let assignment = assign_founders(&mut scenario).map_err(std::io::Error::other)?;
    if assignment.founders.len() != 608
        || assignment.herbivores != 576
        || assignment.vision_0 != 288
        || assignment.vision_1 != 288
    {
        return Err("accepted fixture founder count or balanced assignment mismatch".into());
    }
    let patch_report = if arm.patched() {
        let (patched, report) =
            food_redistribution::redistribute_food(&scenario).map_err(std::io::Error::other)?;
        scenario = patched;
        Some(serde_json::to_value(report)?)
    } else {
        None
    };
    let founder_geometry =
        food_redistribution::founder_geometry(&scenario).map_err(std::io::Error::other)?;
    explicit_profile(&mut scenario, arm);
    let scenario_name = scenario.name.clone();
    let mut world = scenario.build(seed)?;
    configure_agents(&mut world);
    verify_initial(&world, arm, &assignment).map_err(std::io::Error::other)?;
    let initial_state = State::capture(&world);
    let initial_state_value = serde_json::to_value(&initial_state)?;
    let start_stock = ecology_audit::stock(&world);
    let mut previous_stock = start_stock;
    let mut previous_cells = ecology_audit::resource_cells(&world);
    let mut genotype = genotype_audit::Audit::default();
    genotype
        .seed(&initial_state)
        .map_err(|e| std::io::Error::other(format!("genotype audit seed: {e:?}")))?;
    let mut ecology = ecology_audit::Audit::default();
    ecology.seed(&world);
    let founder_ids = assignment
        .founders
        .iter()
        .filter(|f| f.species == Species::Herbivore)
        .map(|f| f.id)
        .collect::<Vec<_>>();
    let mut ancestry = BTreeMap::<usize, Vec<u64>>::new();
    let ancestry_words = founder_ids.len().div_ceil(64);
    for id in assignment.founders.iter().map(|f| f.id) {
        ancestry.insert(id, vec![0; ancestry_words]);
    }
    for (index, id) in founder_ids.iter().enumerate() {
        ancestry.get_mut(id).expect("founder ancestry initialized")[index / 64] |=
            1u64 << (index % 64);
    }
    let tracked_ids = assignment
        .founders
        .iter()
        .map(|f| f.id)
        .collect::<BTreeSet<_>>();
    let mut all_ids = tracked_ids;
    let mut living_individual_ticks = BTreeMap::<String, u64>::new();
    write_line(
        raw,
        raw_bytes,
        &json!({"arm_start":{"schema":"curious-vision-habitat-v1","arm":arm.name(),"seed":seed,"ticks":ticks,"scenario_name":scenario_name,
        "founder_assignment":assignment,"accepted_profile":{"mate_rendezvous":true,"reserve_opportunities":true,"water_residual_enabled":false,
        "genetics_enabled":true,"mutation_per_mille":0,"extra_traits_enabled":arm.extra_traits(),"full_perception_capture":"memory only; compact counts serialized"},
        "habitat":if arm.patched(){"patched_food"}else{"accepted"},"food_patch_transform":patch_report,"t0_founder_geometry":founder_geometry,
        "tick0_state":initial_state_value,"start_food_stock":start_stock.0,"start_water_stock":start_stock.1}}),
        raw_cap,
    )?;

    let mut lineage_events = 0usize;
    for tick in 1..=ticks {
        let before = State::capture(&world);
        let living_pre_by_genotype = living_genotype_counts(&before);
        for (key, count) in &living_pre_by_genotype {
            *living_individual_ticks.entry(key.clone()).or_default() += count;
        }
        let frame_pre = if frame_ticks.contains(&tick) {
            Some(serde_json::to_value(&before)?)
        } else {
            None
        };
        let events = world.tick();
        let after = State::capture(&world);
        let traces = &world.last_decisions;
        if events.len() > MAX_EVENTS_PER_TICK || world.entities.len() > MAX_IDENTITIES {
            return Err(
                format!("event or entity cap exceeded at {} tick {tick}", arm.name()).into(),
            );
        }
        let lineage_here = events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    SimulationEvent::EntityMated { .. }
                        | SimulationEvent::EntityBorn { .. }
                        | SimulationEvent::EntityDied { .. }
                )
            })
            .count();
        if lineage_events.saturating_add(lineage_here) > MAX_LINEAGE_EVENTS
            || genotype.individual_count().saturating_add(
                events
                    .iter()
                    .filter(|e| matches!(e, SimulationEvent::EntityBorn { .. }))
                    .count(),
            ) > MAX_IDENTITIES
        {
            return Err(format!(
                "lineage or retained identity cap exceeded at {} tick {tick}",
                arm.name()
            )
            .into());
        }
        let projected_rows = projected_ecology_issues(&ecology, &world, &events);
        let retained = ecology.reconciliation.len()
            + ecology.replacements.len()
            + ecology.parent_event_issues.len()
            + ecology.unattributed_actor_events.len()
            + ecology.trait_payment_issues.len();
        let cells_now = ecology_audit::resource_cells(&world);
        let replacements = previous_cells
            .keys()
            .chain(cells_now.keys())
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter(|pos| {
                let a = previous_cells.get(pos).copied().unwrap_or("empty");
                let b = cells_now.get(pos).copied().unwrap_or("empty");
                a != b && a != "empty" && b != "empty"
            })
            .count();
        if retained
            .saturating_add(projected_rows)
            .saturating_add(replacements)
            .saturating_add(1)
            > MAX_DETAIL_ROWS
        {
            return Err(format!(
                "detail row cap exceeded before audit mutation at {} tick {tick}",
                arm.name()
            )
            .into());
        }
        let (trace_rows, missing_trace_ids, missing_perception_ids) =
            compact_traces(traces, &before);
        let audited_events = relevant_events(&events, &after)?;
        let trait_state_issues = trait_state_mismatches(&before)
            .into_iter()
            .chain(trait_state_mismatches(&after))
            .collect::<Vec<_>>();
        let trait_event_issues = trait_event_mismatches(&events, &before, &after);
        let full_events = if frame_ticks.contains(&tick) {
            Some(serde_json::to_value(&events)?)
        } else {
            None
        };
        let event_counts =
            events
                .iter()
                .fold(BTreeMap::<String, usize>::new(), |mut map, event| {
                    let val = serde_json::to_value(event).unwrap_or(Value::Null);
                    if let Some(kind) = val.as_object().and_then(|o| o.keys().next()) {
                        *map.entry(kind.clone()).or_default() += 1;
                    }
                    map
                });
        let food_path_samples = if frame_ticks.contains(&tick) {
            let samples = full_food_path_samples(traces);
            let sample_bytes = serde_json::to_vec(&samples)?.len() as u64;
            let projected = frame_sample_bytes
                .checked_add(sample_bytes)
                .ok_or("fixed-frame food path byte count overflow")?;
            if projected > MAX_FRAME_PATH_SAMPLE_BYTES {
                return Err("fixed-frame food path sample cap exceeded".into());
            }
            *frame_sample_bytes = projected;
            Some(samples)
        } else {
            None
        };
        let _ = ecology.observe_tick(tick, previous_stock, &previous_cells, &world, &events);
        let (food_stock, water_stock) = ecology_audit::stock(&world);
        let audit_error = genotype.observe(&before, &after, &events, traces);
        for event in &events {
            if let SimulationEvent::EntityMated {
                parent1,
                parent2,
                child,
                ..
            } = event
            {
                ancestry_mask_union(*parent1, *parent2, *child, &mut ancestry, ancestry_words)
                    .map_err(std::io::Error::other)?;
                all_ids.insert(*child);
            }
        }
        for event in &events {
            if let SimulationEvent::EntityBorn { id, .. } = event {
                all_ids.insert(*id);
            }
        }
        lineage_events += lineage_here;
        let frame=frame_pre.map(|pre|json!({"pre_state":pre,"all_typed_events":full_events,"post_state":serde_json::to_value(&after).unwrap_or(Value::Null)}));
        let row = json!({"arm_tick":{"arm":arm.name(),"tick":tick,"alive_by_species":live_counts(&world),
            "living_pre_by_genotype":living_pre_by_genotype,
            "cumulative_living_genotype_individual_ticks":living_individual_ticks,
            "food_stock":food_stock,"water_stock":water_stock,"event_counts_by_type":event_counts,
            "audit_events":audited_events,"trace_food_path_histograms":trace_rows,
            "food_path_samples":food_path_samples,
            "missing_trace_ids":missing_trace_ids,"missing_full_perception_ids":missing_perception_ids,
            "trait_state_issues":trait_state_issues,"trait_event_issues":trait_event_issues,
            "resource_reconciliation":stock_reconciliation(&ecology),"resource_replacements_cumulative":ecology.replacement_count,
            "genotype_audit_error":audit_error.as_ref().err().map(|e|format!("{e:?}")),"genotype_issues":genotype.issues(),
            "checkpoint_frame":frame}});
        write_line(raw, raw_bytes, &row, raw_cap)?;
        previous_stock = (food_stock, water_stock);
        previous_cells = cells_now;
        if tick == 100 {
            let encoded = serde_json::to_vec(&State::capture(&world))?;
            let decoded: State = serde_json::from_slice(&encoded)?;
            let restored = decoded.restore()?;
            let restored_value = serde_json::to_value(State::capture(&restored))?;
            let roundtrip_ok = restored_value == serde_json::to_value(State::capture(&world))?;
            write_line(
                raw,
                raw_bytes,
                &json!({"checkpoint_restore":{"arm":arm.name(),"tick":100,"state_and_rng_roundtrip_equal":roundtrip_ok,"serialized_bytes":encoded.len()}}),
                raw_cap,
            )?;
            if !roundtrip_ok {
                return Err(
                    format!("checkpoint State/RNG roundtrip mismatch in {}", arm.name()).into(),
                );
            }
            world = restored;
        }
        if audit_error.is_err()
            || !trait_state_issues.is_empty()
            || !trait_event_issues.is_empty()
            || !genotype.issues().is_empty()
            || !ecology.parent_event_issues.is_empty()
            || !ecology.unattributed_actor_events.is_empty()
            || !ecology.trait_payment_issues.is_empty()
            || ecology.replacement_count > 0
            || ecology.food_residual_nonzero_ticks > 0
            || ecology.water_residual_nonzero_ticks > 0
        {
            return Err(format!(
                "observer or resource-flow issue at {} tick {tick}",
                arm.name()
            )
            .into());
        }
    }
    let selection_endpoint =
        primary_and_family_summary(&genotype, &all_ids, &ancestry, &founder_ids)
            .map_err(std::io::Error::other)?;
    let mut genotype_summary = genotype.summary();
    add_exposure_rates(&mut genotype_summary, &living_individual_ticks);
    let result = json!({"selection_endpoint":selection_endpoint,"genotype_summary":genotype_summary,
        "living_genotype_individual_ticks":living_individual_ticks});
    let summary = json!({"arm_complete":{"schema":"curious-vision-habitat-v1","arm":arm.name(),"seed":seed,"ticks_completed":ticks,"initial_state":initial_state_value,
        "genotype_summary":genotype_summary,"selection_endpoint":selection_endpoint,"ecology_audit":ecology.result(&world),"end_state":serde_json::to_value(State::capture(&world))?,
        "living_genotype_individual_ticks":living_individual_ticks,"food_stock_at_start":start_stock.0,"water_stock_at_start":start_stock.1}});
    write_line(raw, raw_bytes, &summary, raw_cap)?;
    Ok(result)
}

fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    const PROTOCOL: &str = include_str!("../docs/VISION_HABITAT_PROTOCOL.md");
    if !PROTOCOL
        .lines()
        .any(|line| line.contains("Status: FROZEN BY ROOT"))
    {
        return Err(
            "vision habitat protocol is not frozen by root; refusing World execution".into(),
        );
    }
    let args: Vec<_> = env::args().collect();
    if args.len() != 5 {
        return Err(
            "usage: vision_habitat_lab PHASE SEED ACCEPTED_SCENARIO.json NEW_OUTPUT.jsonl".into(),
        );
    }
    let phase = args[1].as_str();
    let seed = args[2].parse::<u64>()?;
    match phase {
        "engineering" if seed == ENGINEERING_SEED => {}
        "formal" if FORMAL_SEEDS.contains(&seed) => {
            if !PROTOCOL.contains("`55001`, `55002`, `55003`, and `55004`") {
                return Err(
                    "formal seed allowlist in frozen protocol does not match driver".into(),
                );
            }
        }
        "engineering" => return Err("engineering role only accepts seen seed 123".into()),
        "formal" => return Err("formal seed is outside frozen four-seed allowlist".into()),
        _ => return Err("phase must be engineering or formal".into()),
    }
    let (ticks, frame_ticks) = if phase == "engineering" {
        (ENGINEERING_TICKS, [1usize, 100, 200])
    } else {
        (FORMAL_TICKS, [1usize, 100, 500])
    };
    let raw_cap = if phase == "engineering" {
        MAX_RAW_BYTES
    } else {
        480 * 1024 * 1024
    };
    let row_forecast_bytes = compact_trace_row_forecast_bytes();
    let engineering_forecast =
        capture_size_forecast(row_forecast_bytes).ok_or("capture-size forecast overflow")?;
    if phase == "engineering" && engineering_forecast > MAX_RAW_BYTES {
        return Err("engineering compact capture forecast exceeds raw cap".into());
    }
    let scenario_bytes = fs::read(&args[3])?;
    let output_path = Path::new(&args[4]);
    let output_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output_path)?;
    let mut raw = BufWriter::new(output_file);
    let mut bytes = 0u64;
    let arms = [
        Arm::AcceptedNeutral,
        Arm::AcceptedExpressed,
        Arm::PatchedNeutral,
        Arm::PatchedExpressed,
    ];
    write_line(
        &mut raw,
        &mut bytes,
        &json!({"vision_habitat_header":{"schema":"curious-vision-habitat-v1","phase":phase,"seed":seed,"ticks_per_cell":ticks,"cells":arms.iter().map(|arm|arm.name()).collect::<Vec<_>>(),"formal_seed_allowlist":FORMAL_SEEDS,"claim_scope":"single seen-seed engineering interaction and conditional paired descriptive habitat contrast; not causal vision utility or general fitness","raw_cap_bytes_per_quad":raw_cap,"engineering_capture_forecast":{"prior_raw_bytes":PRIOR_RAW_BYTES,"prior_ticks":PRIOR_TICKS,"prior_trace_rows":PRIOR_TRACE_ROWS,"projected_trace_rows":PRIOR_TRACE_ROWS*2,"compact_trace_row_estimate_bytes":row_forecast_bytes,"fixed_frame_food_path_sample_budget":MAX_FRAME_PATH_SAMPLE_BYTES,"forecast_raw_bytes":engineering_forecast,"margin_per_mille":100},"fixed_frames":if phase=="formal"{vec![0,1,100,500]}else{vec![0,1,100,200]},"hard_limits":{"identities":MAX_IDENTITIES,"events_per_tick":MAX_EVENTS_PER_TICK,"lineage_events_per_cell":MAX_LINEAGE_EVENTS,"detail_rows_per_cell":MAX_DETAIL_ROWS}}}),
        raw_cap,
    )?;
    let mut results = BTreeMap::<String, Value>::new();
    let mut frame_sample_bytes = 0u64;
    for arm in arms {
        let result = run_arm(
            &scenario_bytes,
            arm,
            seed,
            ticks,
            &frame_ticks,
            raw_cap,
            &mut frame_sample_bytes,
            &mut raw,
            &mut bytes,
        )?;
        results.insert(arm.name().to_string(), result);
    }
    let arm_delta = |neutral: &str, expressed: &str| {
        let n = &results[neutral]["selection_endpoint"];
        let e = &results[expressed]["selection_endpoint"];
        paired_change_delta(
            e["founder_vision_1_frequency"].as_f64(),
            e["living_descendant_breeder_vision_1_frequency"].as_f64(),
            n["founder_vision_1_frequency"].as_f64(),
            n["living_descendant_breeder_vision_1_frequency"].as_f64(),
        )
    };
    let accepted_delta = arm_delta("accepted_neutral", "accepted_expressed");
    let patched_delta = arm_delta("patched_neutral", "patched_expressed");
    let interaction = patched_delta
        .zip(accepted_delta)
        .map(|(patched, accepted)| patched - accepted);
    let raw_before_footer = bytes;
    write_line(
        &mut raw,
        &mut bytes,
        &json!({"vision_habitat_complete":{"status":"capture_complete","phase":phase,"seed":seed,"ticks_per_cell":ticks,"paired_seed_interaction_patched_minus_accepted":interaction,"accepted_expressed_minus_neutral":accepted_delta,"patched_expressed_minus_neutral":patched_delta,"cells":results,"raw_bytes_before_footer":raw_before_footer}}),
        raw_cap,
    )?;
    raw.get_ref().sync_all()?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("vision_habitat_lab: {error}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod pure_tests {
    use super::*;

    #[test]
    fn fixation_is_valid_and_only_zero_breeder_denominator_is_not_estimable() {
        assert_eq!(allele_frequency(0, 12), Some(0.0));
        assert_eq!(allele_frequency(12, 12), Some(1.0));
        assert_eq!(allele_frequency(0, 0), None);
        assert_eq!(within_arm_change(Some(0.5), Some(1.0)), Some(0.5));
        assert_eq!(within_arm_change(Some(0.5), None), None);
    }

    #[test]
    fn paired_delta_uses_founder_terms_and_preserves_estimable_fixation() {
        assert_eq!(
            paired_change_delta(Some(0.5), Some(1.0), Some(0.5), Some(0.25)),
            Some(0.75)
        );
        assert_eq!(
            paired_change_delta(Some(0.5), None, Some(0.5), Some(0.25)),
            None
        );
    }

    #[test]
    fn founder_ancestor_masks_union_without_counting_descendants_as_families() {
        let mut ancestry =
            BTreeMap::from([(10, vec![0b001]), (11, vec![0b010]), (12, vec![0b100])]);
        ancestry_mask_union(10, 11, 20, &mut ancestry, 1).unwrap();
        ancestry_mask_union(20, 12, 21, &mut ancestry, 1).unwrap();
        assert_eq!(ancestry[&20], vec![0b011]);
        assert_eq!(ancestry[&21], vec![0b111]);
    }

    #[test]
    fn genotype_rates_use_living_individual_ticks_not_trace_rows() {
        let mut summary = json!({
            "genotypes": {
                "Herbivore:m0v1s0": {
                    "vision_requested": 6,
                    "vision_paid": 4,
                    "vision_unpaid": 2,
                    "plant_food": 12,
                    "corpse_food": 3,
                    "water": 8,
                    "trace_ticks": 1
                }
            }
        });
        add_exposure_rates(
            &mut summary,
            &BTreeMap::from([("Herbivore:m0v1s0".into(), 2)]),
        );
        let row = &summary["genotypes"]["Herbivore:m0v1s0"];
        assert_eq!(row["living_genotype_individual_ticks"], 2);
        assert_eq!(row["vision_requested_per_living_individual_tick"], 3.0);
        assert_eq!(row["plant_food_per_living_individual_tick"], 6.0);
        assert_eq!(row["trace_ticks"], 1);
    }

    #[test]
    fn registered_roles_and_tick_schedules_are_fixed() {
        assert_eq!(FORMAL_SEEDS, [55001, 55002, 55003, 55004]);
        assert_eq!(ENGINEERING_TICKS, 200);
        assert_eq!(FORMAL_TICKS, 500);
        assert_eq!(Arm::AcceptedNeutral.name(), "accepted_neutral");
        assert_eq!(Arm::PatchedExpressed.name(), "patched_expressed");
    }

    #[test]
    fn food_paths_at_six_and_seven_are_split_by_kind_and_repeated_entries_count() {
        use curious::map::movement::{Direction, Steps};
        let mut perception = Perception::empty();
        perception.add_food(5, false, Steps::new(vec![Direction::Right; 6]));
        perception.add_food(4, false, Steps::new(vec![Direction::Left; 6]));
        perception.add_food(5, false, Steps::new(vec![Direction::Right; 7]));
        perception.add_food(5, true, Steps::new(vec![Direction::Left; 6]));
        perception.add_food(5, true, Steps::new(vec![Direction::Left; 7]));
        let counts = food_trace_counts(Some(&perception)).unwrap();
        assert_eq!(counts.0, vec![[6, 2], [7, 1]]);
        assert_eq!(counts.1, vec![[6, 1], [7, 1]]);
        assert_eq!(path_histogram_metrics(&counts.0), (2, 1, true));
        assert_eq!(path_histogram_metrics(&counts.1), (1, 1, true));
        assert_eq!(path_histogram_metrics(&[]), (0, 0, false));
        assert!(food_trace_counts(None).is_none());
    }

    #[test]
    fn missing_trace_ids_remain_explicit_and_missing_perception_is_unknown() {
        let expected = [1, 2, 3];
        let no_traces: Vec<DecisionTrace> = Vec::new();
        assert_eq!(missing_trace_ids(&expected, &no_traces), expected);
        assert!(food_trace_counts(None).is_none());
        let empty = Perception::empty();
        let known_empty = food_trace_counts(Some(&empty)).unwrap();
        assert!(known_empty.0.is_empty());
        assert!(known_empty.1.is_empty());
    }

    #[test]
    fn compact_trace_capture_forecast_uses_prior_raw_and_actual_trace_counts() {
        let row_bytes = compact_trace_row_forecast_bytes();
        let forecast = capture_size_forecast(row_bytes).unwrap();
        assert!(row_bytes > 0);
        assert_eq!(PRIOR_TRACE_ROWS, 258_470);
        assert_eq!(PRIOR_RAW_BYTES, 80_367_647);
        assert!(forecast < MAX_RAW_BYTES);
        assert_eq!(forecast, 230_519_069);
    }
}
