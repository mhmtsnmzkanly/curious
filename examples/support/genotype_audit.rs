//! Bounded, read-only genotype lifecycle accounting for handoff audits.
use curious::{
    entity::{decision::DecisionTrace, genetics::Genome, species::Species},
    simulation::{SimulationEvent, checkpoint::State},
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const MAX_IDS: usize = 50_000;
pub const MAX_ISSUES: usize = 2_000;

#[derive(Clone, Debug)]
pub struct Individual {
    pub species: Species,
    pub genome: Genome,
    pub generation: usize,
    pub parents: Vec<usize>,
    pub founder: bool,
    pub born: bool,
    pub lineage_known: bool,
    pub mated: bool,
    pub alive: bool,
    pub vision_range: usize,
    pub stamina: usize,
    pub max_stamina: usize,
    pub totals: Totals,
}

#[derive(Clone, Debug, Default)]
pub struct Totals {
    pub vision_requested: u64,
    pub vision_paid: u64,
    pub stamina_requested: u64,
    pub stamina_paid: u64,
    pub plant_food: u64,
    pub corpse_food: u64,
    pub water: u64,
    pub observed_plant: u64,
    pub observed_corpse: u64,
    pub trace_ticks: u64,
    pub missing_trace_ticks: u64,
    pub births: u64,
    pub deaths: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuditError {
    AlreadySeeded,
    NotSeeded,
    IdBudget,
    IssueBudget,
    InvalidGenome(usize),
}

#[derive(Default)]
pub struct Audit {
    individuals: BTreeMap<usize, Individual>,
    issues: Vec<Value>,
    mating_events: u64,
    birth_events: u64,
    seeded: bool,
}

fn add_issue(issues: &mut Vec<Value>, item: Value) -> Result<(), AuditError> {
    if issues.len() >= MAX_ISSUES {
        return Err(AuditError::IssueBudget);
    }
    issues.push(item);
    Ok(())
}
fn genotype(species: &Species, genome: &Genome) -> String {
    format!(
        "{:?}:m{}v{}s{}",
        species, genome.mobility, genome.vision, genome.stamina
    )
}
fn allele_transition(source: u8, child: u8, mutation_enabled: bool) -> bool {
    source == child
        || (mutation_enabled
            && match source {
                0 | 2 => child == 1,
                1 => child == 0 || child == 2,
                _ => false,
            })
}
fn transmission_match(
    child: &Genome,
    parent: &Genome,
    genetics: &curious::entity::genetics::Genetics,
) -> (bool, bool) {
    let mutated =
        |a, b| genetics.enabled && genetics.mutation_per_mille > 0 && allele_transition(a, b, true);
    let exact = child == parent;
    if !genetics.enabled {
        return (child == parent, false);
    }
    let can_match = allele_transition(
        parent.mobility,
        child.mobility,
        genetics.mutation_per_mille > 0,
    ) && (child.vision == parent.vision
        || (genetics.extra_traits_enabled && mutated(parent.vision, child.vision)))
        && (child.stamina == parent.stamina
            || (genetics.extra_traits_enabled && mutated(parent.stamina, child.stamina)));
    (exact, can_match && !exact)
}
fn from_state(ind: &curious::simulation::checkpoint::Individual, founder: bool) -> Individual {
    Individual {
        species: ind.species.clone(),
        genome: ind.genome.clone(),
        generation: ind.generation,
        parents: ind.parents.clone(),
        founder,
        born: false,
        lineage_known: founder,
        mated: false,
        alive: ind.life.is_alive()
            && !matches!(
                ind.phase,
                curious::entity::phase::EntityPhase::Corpse { .. }
                    | curious::entity::phase::EntityPhase::Removed
            ),
        vision_range: ind.life.vision_range,
        stamina: ind.life.stamina,
        max_stamina: ind.life.max_stamina,
        totals: Totals::default(),
    }
}

impl Audit {
    pub fn individual(&self, id: usize) -> Option<&Individual> {
        self.individuals.get(&id)
    }

    pub fn issues(&self) -> &[Value] {
        &self.issues
    }

    pub fn individual_count(&self) -> usize {
        self.individuals.len()
    }

    pub fn mating_event_count(&self) -> u64 {
        self.mating_events
    }

    pub fn seed(&mut self, state: &State) -> Result<(), AuditError> {
        if self.seeded {
            return Err(AuditError::AlreadySeeded);
        }
        if state.entities.len() > MAX_IDS {
            return Err(AuditError::IdBudget);
        }
        for entity in &state.entities {
            if entity.genome.validate().is_err() {
                return Err(AuditError::InvalidGenome(entity.id));
            }
        }
        let mut next = BTreeMap::new();
        for entity in &state.entities {
            next.insert(entity.id, from_state(entity, true));
        }
        self.individuals = next;
        self.seeded = true;
        Ok(())
    }

    /// `before` must be the actual pre-tick checkpoint and `after` the post-tick checkpoint.
    /// This observer never supplies information to a policy or World.
    pub fn observe(
        &mut self,
        before: &State,
        after: &State,
        events: &[SimulationEvent],
        traces: &[DecisionTrace],
    ) -> Result<(), AuditError> {
        if !self.seeded {
            return Err(AuditError::NotSeeded);
        }
        for entity in after.entities.iter().chain(before.entities.iter()) {
            if entity.genome.validate().is_err() {
                return Err(AuditError::InvalidGenome(entity.id));
            }
        }
        let new_ids = after
            .entities
            .iter()
            .filter(|individual| !self.individuals.contains_key(&individual.id))
            .count();
        if self.individuals.len().saturating_add(new_ids) > MAX_IDS {
            return Err(AuditError::IdBudget);
        }
        let mut ledger = self.individuals.clone();
        let mut issues = self.issues.clone();
        let mut mating_events = self.mating_events;
        let mut birth_events = self.birth_events;
        let before_by: BTreeMap<_, _> = before.entities.iter().map(|i| (i.id, i)).collect();
        let after_by: BTreeMap<_, _> = after.entities.iter().map(|i| (i.id, i)).collect();
        let trace_by: BTreeMap<_, _> = traces.iter().map(|t| (t.id, t)).collect();
        for event in events {
            match event {
                SimulationEvent::EntityMated {
                    parent1,
                    parent2,
                    child,
                    ..
                } => {
                    mating_events += 1;
                    for id in [parent1, parent2] {
                        if let Some(p) = ledger.get_mut(id) {
                            p.mated = true;
                        } else {
                            add_issue(&mut issues, json!({"kind":"missing_parent","id":id}))?;
                        }
                    }
                    let child_state = after_by.get(child).copied();
                    if let Some(c) = child_state {
                        let expected = ledger
                            .get(parent1)
                            .zip(ledger.get(parent2))
                            .map(|(a, b)| a.generation.max(b.generation) + 1);
                        let parent_match = c.parents.len() == 2
                            && c.parents.contains(parent1)
                            && c.parents.contains(parent2);
                        if !parent_match {
                            add_issue(
                                &mut issues,
                                json!({"kind":"wrong_parent_child","child":child,"parents":c.parents}),
                            )?;
                        }
                        if expected.is_some_and(|g| g != c.generation) {
                            add_issue(
                                &mut issues,
                                json!({"kind":"generation_mismatch","child":child,"expected":expected,"actual":c.generation}),
                            )?;
                        }
                        let settings = &before.genetics;
                        let (matches_parent1, mutation1) = if settings.enabled {
                            ledger.get(parent1).map_or((false, false), |p| {
                                transmission_match(&c.genome, &p.genome, settings)
                            })
                        } else {
                            (false, false)
                        };
                        let (matches_parent2, mutation2) =
                            ledger.get(parent2).map_or((false, false), |p| {
                                transmission_match(&c.genome, &p.genome, settings)
                            });
                        let allowed = matches_parent1 || matches_parent2 || mutation1 || mutation2;
                        if !allowed {
                            add_issue(
                                &mut issues,
                                json!({"kind":"unexpected_genome_transmission","child":child,"genetics_enabled":settings.enabled,"mutation_per_mille":settings.mutation_per_mille,"extra_traits_enabled":settings.extra_traits_enabled}),
                            )?;
                        } else if !matches_parent1 && !matches_parent2 {
                            add_issue(
                                &mut issues,
                                json!({"kind":"possible_mutated_genome_transmission","child":child,"mutation_per_mille":settings.mutation_per_mille}),
                            )?;
                        }
                        let parent_match = c.parents.len() == 2
                            && c.parents.contains(parent1)
                            && c.parents.contains(parent2);
                        let generation_match = expected.is_some_and(|g| g == c.generation);
                        let child_entry =
                            ledger.entry(*child).or_insert_with(|| from_state(c, false));
                        child_entry.lineage_known = parent_match && generation_match;
                    } else {
                        add_issue(
                            &mut issues,
                            json!({"kind":"missing_child_state","child":child}),
                        )?;
                    }
                }
                SimulationEvent::EntityBorn { id, species, .. } => {
                    birth_events += 1;
                    let Some(c) = after_by.get(id).copied() else {
                        add_issue(&mut issues, json!({"kind":"missing_birth_state","id":id}))?;
                        continue;
                    };
                    if c.species != *species {
                        add_issue(
                            &mut issues,
                            json!({"kind":"birth_species_mismatch","id":id}),
                        )?;
                    }
                    if !ledger.contains_key(id) {
                        ledger.insert(*id, from_state(c, false));
                        add_issue(
                            &mut issues,
                            json!({"kind":"birth_without_mating_event","id":id}),
                        )?;
                    }
                    if let Some(ind) = ledger.get_mut(id) {
                        ind.born = true;
                        ind.totals.births += 1;
                        if !ind.lineage_known {
                            add_issue(
                                &mut issues,
                                json!({"kind":"birth_lineage_unknown","id":id}),
                            )?;
                        }
                    }
                }
                SimulationEvent::EntityDied { id, .. } => {
                    if let Some(ind) = ledger.get_mut(id) {
                        ind.alive = false;
                        ind.totals.deaths += 1;
                    } else {
                        add_issue(&mut issues, json!({"kind":"missing_death_actor","id":id}))?;
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
                        add_issue(
                            &mut issues,
                            json!({"kind":"payment_exceeds_request","id":id}),
                        )?;
                    }
                    if let Some(ind) = ledger.get_mut(id) {
                        let t = &mut ind.totals;
                        t.vision_requested += *vision_cost as u64;
                        t.vision_paid += *vision_paid as u64;
                        t.stamina_requested += *stamina_cost as u64;
                        t.stamina_paid += *stamina_paid as u64;
                    } else {
                        add_issue(
                            &mut issues,
                            json!({"kind":"missing_metabolism_actor","id":id}),
                        )?;
                    }
                }
                SimulationEvent::EntityAte {
                    id,
                    corpse_id,
                    amount,
                    ..
                } => {
                    if let Some(i) = ledger.get_mut(id) {
                        if corpse_id.is_some() {
                            i.totals.corpse_food += *amount as u64
                        } else {
                            i.totals.plant_food += *amount as u64
                        }
                    } else {
                        add_issue(&mut issues, json!({"kind":"missing_food_actor","id":id}))?;
                    }
                }
                SimulationEvent::EntityDrank { id, amount, .. } => {
                    if let Some(i) = ledger.get_mut(id) {
                        i.totals.water += *amount as u64
                    } else {
                        add_issue(&mut issues, json!({"kind":"missing_water_actor","id":id}))?;
                    }
                }
                _ => {}
            }
        }
        for (id, old) in &before_by {
            let Some(ind) = ledger.get_mut(id) else {
                continue;
            };
            if let Some(trace) = trace_by.get(id) {
                ind.totals.trace_ticks += 1;
                if let Some(view) = &trace.perception {
                    for f in &view.foods {
                        if f.is_corpse {
                            ind.totals.observed_corpse += 1
                        } else {
                            ind.totals.observed_plant += 1
                        }
                    }
                } else {
                    ind.totals.missing_trace_ticks += 1;
                }
            } else {
                ind.totals.missing_trace_ticks += 1;
            }
            if !after_by.contains_key(id) {
                if ind.alive
                    && !events.iter().any(|event| {
                        matches!(event, SimulationEvent::EntityDied { id: dead, .. } if dead == id)
                    })
                {
                    add_issue(
                        &mut issues,
                        json!({"kind":"removed_without_death_event","id":id}),
                    )?;
                }
                ind.alive = false;
            }
            let _ = old;
        }
        for (id, current) in after_by {
            if !ledger.contains_key(&id) {
                ledger.insert(id, from_state(current, false));
                add_issue(
                    &mut issues,
                    json!({"kind":"new_id_without_birth_event","id":id}),
                )?;
            }
            if let Some(x) = ledger.get_mut(&id) {
                x.alive = current.life.is_alive()
                    && !matches!(
                        current.phase,
                        curious::entity::phase::EntityPhase::Corpse { .. }
                            | curious::entity::phase::EntityPhase::Removed
                    );
                x.vision_range = current.life.vision_range;
                x.stamina = current.life.stamina;
                x.max_stamina = current.life.max_stamina;
            }
        }
        if ledger.len() > MAX_IDS {
            return Err(AuditError::IdBudget);
        }
        self.individuals = ledger;
        self.issues = issues;
        self.mating_events = mating_events;
        self.birth_events = birth_events;
        Ok(())
    }

    pub fn summary(&self) -> Value {
        let mut groups: BTreeMap<String, Value> = BTreeMap::new();
        let mut allele: BTreeMap<String, [u64; 10]> = BTreeMap::new();
        let mut count_allele = |cohort: String, i: &Individual| {
            let x = allele
                .entry(format!("{:?}:{cohort}", i.species))
                .or_default();
            x[0] += 1;
            x[1 + i.genome.mobility as usize] += 1;
            x[4 + i.genome.vision as usize] += 1;
            x[7 + i.genome.stamina as usize] += 1;
        };
        for i in self.individuals.values() {
            let key = genotype(&i.species, &i.genome);
            let g=groups.entry(key).or_insert(json!({"individuals":0,"living":0,"founders":0,"descendants":0,"living_descendant_breeders":0,"births":0,"deaths":0,"vision_requested":0,"vision_paid":0,"vision_unpaid":0,"stamina_requested":0,"stamina_paid":0,"stamina_unpaid":0,"expressed_vision_range_sum":0,"expressed_stamina_sum":0,"plant_food":0,"corpse_food":0,"water":0,"observed_plant_encounters":0,"observed_corpse_encounters":0,"trace_ticks":0,"missing_trace_ticks":0}));
            if i.founder {
                count_allele("founders".into(), i);
            }
            if i.alive {
                count_allele("living".into(), i);
            }
            if i.born && i.lineage_known {
                count_allele("descendants".into(), i);
                if i.alive {
                    count_allele("living_descendants".into(), i);
                    if i.mated {
                        count_allele("living_descendant_breeders".into(), i);
                    }
                }
            }
            for field in [
                "individuals",
                "living",
                "founders",
                "descendants",
                "living_descendant_breeders",
                "births",
                "deaths",
                "vision_requested",
                "vision_paid",
                "vision_unpaid",
                "stamina_requested",
                "stamina_paid",
                "stamina_unpaid",
                "expressed_vision_range_sum",
                "expressed_stamina_sum",
                "plant_food",
                "corpse_food",
                "water",
                "observed_plant_encounters",
                "observed_corpse_encounters",
                "trace_ticks",
                "missing_trace_ticks",
            ] {
                let add = match field {
                    "individuals" => 1,
                    "living" => i.alive as u64,
                    "founders" => i.founder as u64,
                    "descendants" => (i.born && i.lineage_known) as u64,
                    "living_descendant_breeders" => {
                        (i.born && i.lineage_known && i.alive && i.mated) as u64
                    }
                    "births" => i.totals.births,
                    "deaths" => i.totals.deaths,
                    "vision_requested" => i.totals.vision_requested,
                    "vision_paid" => i.totals.vision_paid,
                    "vision_unpaid" => i
                        .totals
                        .vision_requested
                        .saturating_sub(i.totals.vision_paid),
                    "stamina_requested" => i.totals.stamina_requested,
                    "stamina_paid" => i.totals.stamina_paid,
                    "stamina_unpaid" => i
                        .totals
                        .stamina_requested
                        .saturating_sub(i.totals.stamina_paid),
                    "expressed_vision_range_sum" => i.vision_range as u64,
                    "expressed_stamina_sum" => i.stamina as u64,
                    "plant_food" => i.totals.plant_food,
                    "corpse_food" => i.totals.corpse_food,
                    "water" => i.totals.water,
                    "observed_plant_encounters" => i.totals.observed_plant,
                    "observed_corpse_encounters" => i.totals.observed_corpse,
                    "trace_ticks" => i.totals.trace_ticks,
                    _ => i.totals.missing_trace_ticks,
                };
                g[field] = json!(g[field].as_u64().unwrap_or(0) + add);
            }
        }
        let allele_frequencies: BTreeMap<_, _> = allele
            .into_iter()
            .map(|(key, counts)| {
                let n = counts[0];
                let trait_counts = |offset: usize| {
                    let mut counts_by_value = serde_json::Map::new();
                    let mut frequencies = serde_json::Map::new();
                    for value in 0..=2 {
                        let count = counts[offset + value];
                        counts_by_value.insert(value.to_string(), json!(count));
                        frequencies.insert(
                            value.to_string(),
                            if n == 0 { Value::Null } else { json!(count as f64 / n as f64) },
                        );
                    }
                    json!({"counts":counts_by_value,"frequencies":frequencies})
                };
                (
                    key,
                    json!({"individuals":n,"mobility":trait_counts(1),"vision":trait_counts(4),"stamina":trait_counts(7)}),
                )
            })
            .collect();
        json!({"genotypes":groups,"allele_frequencies":allele_frequencies,"issues":self.issues,"mating_events":self.mating_events,"birth_events":self.birth_events,"coverage":"missing trace ticks are unknown, not zero encounters"})
    }
}
