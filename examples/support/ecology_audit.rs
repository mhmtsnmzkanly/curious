//! Measurement helpers for the bounded ecology candidate.
//!
//! This module records event-backed quantities and explicit residuals. It does
//! not infer basal energy expenditure or attribute unexplained map changes.
use curious::{
    entity::species::Species,
    environment::ResourceKind,
    map::{cell::Cell, movement::Position},
    simulation::SimulationEvent,
    world::{EntitySlot, World},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const MAX_DETAIL_ROWS: usize = 2_000;

#[derive(Clone)]
pub struct Individual {
    pub id: usize,
    pub species: Species,
    pub generation: Option<usize>,
    pub parents: Vec<usize>,
    pub founder: bool,
    pub death_cause: Option<String>,
    pub mated: bool,
    pub births: usize,
    pub offspring: Vec<usize>,
    pub snapshot: Value,
    pub last_body_observation: Value,
    pub last_body_observation_tick: usize,
}

pub fn profile(slot: &EntitySlot, founder: bool) -> Individual {
    let life = slot.base.life();
    Individual {
        id: slot.id,
        species: slot.base.species(),
        generation: Some(slot.generation),
        parents: slot.parents.clone(),
        founder,
        death_cause: None,
        mated: false,
        births: 0,
        offspring: Vec::new(),
        snapshot: json!({
            "id": slot.id,
            "species": format!("{:?}", slot.base.species()),
            "generation": slot.generation,
            "parents": slot.parents,
            "genotype": slot.genome,
            "phenotype": {
                "speed": life.speed,
                "move_energy_cost": life.move_energy_cost,
                "vision_range": life.vision_range,
                "max_stamina": life.max_stamina,
                "vision_metabolic_cost_requested": life.vision_metabolic_cost,
                "stamina_metabolic_cost_requested": life.stamina_metabolic_cost
            }
        }),
        last_body_observation: json!({"energy":life.energy,"water":life.water,"health":life.health}),
        last_body_observation_tick: 0,
    }
}

pub fn stock(world: &World) -> (usize, usize) {
    let (mut food, mut water) = (0, 0);
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

pub fn resource_cells(world: &World) -> BTreeMap<(isize, isize), &'static str> {
    let mut cells = BTreeMap::new();
    for y in world.map.min_y..=world.map.max_y {
        for x in world.map.min_x..=world.map.max_x {
            let pos = Position::new(x, y);
            match world.map.cell(pos) {
                Some(Cell::Food { .. }) => {
                    cells.insert((x, y), "food");
                }
                Some(Cell::Water { .. }) => {
                    cells.insert((x, y), "water");
                }
                _ => {}
            }
        }
    }
    cells
}

fn species_key(species: Species) -> String {
    format!("{:?}", species)
}
fn add(map: &mut BTreeMap<String, usize>, key: String, amount: usize) {
    *map.entry(key).or_default() += amount;
}

#[derive(Default)]
pub struct Audit {
    pub ledger: BTreeMap<usize, Individual>,
    pub eaten_plant_by_species: BTreeMap<String, usize>,
    pub eaten_corpse_by_species: BTreeMap<String, usize>,
    pub drank_by_species: BTreeMap<String, usize>,
    pub movement_by_species: BTreeMap<String, Value>,
    pub traits_by_species: BTreeMap<String, Value>,
    pub births_by_generation: BTreeMap<usize, usize>,
    pub unknown_generation_births: usize,
    pub deaths_by_cause: BTreeMap<String, usize>,
    pub resource_renewals: BTreeMap<String, usize>,
    pub food_meals: usize,
    pub corpse_meals: usize,
    pub drinks: usize,
    pub reconciliation: Vec<Value>,
    pub reconciliation_ticks: usize,
    pub food_residual_nonzero_ticks: usize,
    pub water_residual_nonzero_ticks: usize,
    pub food_residual_total: i128,
    pub water_residual_total: i128,
    pub food_unexplained_loss_total: u128,
    pub food_unexplained_gain_total: u128,
    pub water_unexplained_loss_total: u128,
    pub water_unexplained_gain_total: u128,
    pub first_reconciliation_anomaly: Option<Value>,
    pub last_reconciliation_anomaly: Option<Value>,
    pub replacements: Vec<Value>,
    pub replacement_count: usize,
    pub first_replacement: Option<Value>,
    pub last_replacement: Option<Value>,
    pub parent_event_issues: Vec<Value>,
    pub unattributed_actor_events: Vec<Value>,
    pub trait_payment_issues: Vec<Value>,
    pub parented_births: BTreeSet<usize>,
    pub bred_ids: BTreeSet<usize>,
}

impl Audit {
    pub fn seed(&mut self, world: &World) {
        for slot in &world.entities {
            self.ledger.insert(slot.id, profile(slot, true));
        }
    }

    pub fn observe_tick(
        &mut self,
        tick: usize,
        previous: (usize, usize),
        previous_cells: &BTreeMap<(isize, isize), &'static str>,
        world: &World,
        events: &[SimulationEvent],
    ) -> ((usize, usize), BTreeMap<(isize, isize), &'static str>) {
        let (mut food_renewed, mut water_renewed, mut plant_meals, mut drinks) = (0, 0, 0, 0);
        for event in events {
            match event {
                SimulationEvent::EntityMated {
                    parent1,
                    parent2,
                    child,
                    ..
                } => {
                    self.bred_ids.extend([*parent1, *parent2]);
                    if let Some(x) = self.ledger.get_mut(parent1) {
                        x.mated = true;
                        x.births += 1;
                        x.offspring.push(*child);
                    }
                    if let Some(x) = self.ledger.get_mut(parent2) {
                        x.mated = true;
                        x.births += 1;
                        x.offspring.push(*child);
                    }
                    let parents_known =
                        self.ledger.contains_key(parent1) && self.ledger.contains_key(parent2);
                    let expected_generation = self
                        .ledger
                        .get(parent1)
                        .and_then(|x| x.generation)
                        .zip(self.ledger.get(parent2).and_then(|x| x.generation))
                        .map(|(a, b)| a.max(b) + 1);
                    if !parents_known {
                        self.parent_event_issues.push(json!({"tick":tick,"kind":"unknown_parent","parent1":parent1,"parent2":parent2,"child":child}));
                    }
                    let slot = world.entities.iter().find(|s| s.id == *child);
                    if let Some(slot) = slot {
                        if expected_generation
                            .is_some_and(|generation| slot.generation != generation)
                            || slot.parents.len() != 2
                            || !slot.parents.contains(parent1)
                            || !slot.parents.contains(parent2)
                        {
                            self.parent_event_issues.push(json!({"tick":tick,"kind":"parent_ledger_mismatch","child":child,"expected_generation":expected_generation,"slot_generation":slot.generation,"slot_parents":slot.parents}));
                        }
                        self.ledger.insert(*child, profile(slot, false));
                    } else {
                        self.parent_event_issues.push(
                            json!({"tick":tick,"kind":"mated_child_slot_missing","child":child}),
                        );
                    }
                    self.parented_births.insert(*child);
                }
                SimulationEvent::EntityBorn { id, species, .. } => {
                    if !self.ledger.contains_key(id) {
                        if let Some(slot) = world.entities.iter().find(|s| s.id == *id) {
                            self.ledger.insert(*id, profile(slot, false));
                        }
                    }
                    let generation = self.ledger.get(id).and_then(|x| x.generation);
                    if let Some(item) = self.ledger.get(id) {
                        if item.species != *species {
                            self.parent_event_issues.push(json!({"tick":tick,"kind":"birth_species_mismatch","child":id,"event_species":format!("{:?}",species),"ledger_species":format!("{:?}",item.species)}));
                        }
                    }
                    if let Some(generation) = generation {
                        *self.births_by_generation.entry(generation).or_default() += 1;
                    } else {
                        self.unknown_generation_births += 1;
                    }
                    if !self.parented_births.contains(id) {
                        self.parent_event_issues.push(json!({"tick":tick,"kind":"birth_without_parent_event","child":id,"species":format!("{:?}",species)}));
                    }
                    if let Some(slot) = world.entities.iter().find(|s| s.id == *id) {
                        self.ledger
                            .entry(*id)
                            .or_insert_with(|| profile(slot, false));
                    }
                }
                SimulationEvent::EntityDied {
                    id, cause, species, ..
                } => {
                    let label = format!(
                        "{:?}",
                        cause.unwrap_or(curious::simulation::event::DeathCause::Unknown)
                    );
                    add(
                        &mut self.deaths_by_cause,
                        format!("{}:{label}", species_key(species.clone())),
                        1,
                    );
                    if let Some(x) = self.ledger.get_mut(id) {
                        x.death_cause = Some(label);
                    }
                }
                SimulationEvent::EntityAte {
                    id,
                    corpse_id,
                    amount,
                    ..
                } => {
                    self.corpse_meals += if corpse_id.is_some() { *amount } else { 0 };
                    self.food_meals += if corpse_id.is_none() { *amount } else { 0 };
                    if let Some(ind) = self.ledger.get(id) {
                        add(
                            if corpse_id.is_some() {
                                &mut self.eaten_corpse_by_species
                            } else {
                                &mut self.eaten_plant_by_species
                            },
                            species_key(ind.species.clone()),
                            *amount,
                        );
                    } else {
                        self.unattributed_actor_events.push(json!({"tick":tick,"event":"EntityAte","id":id,"amount":amount,"corpse_id":corpse_id}));
                    }
                    if corpse_id.is_none() {
                        plant_meals += amount;
                    }
                }
                SimulationEvent::EntityDrank { id, amount, .. } => {
                    self.drinks += amount;
                    drinks += amount;
                    if let Some(ind) = self.ledger.get(id) {
                        add(
                            &mut self.drank_by_species,
                            species_key(ind.species.clone()),
                            *amount,
                        );
                    } else {
                        self.unattributed_actor_events.push(
                            json!({"tick":tick,"event":"EntityDrank","id":id,"amount":amount}),
                        );
                    }
                }
                SimulationEvent::EntityMoved {
                    id, energy_cost, ..
                } => {
                    if let Some(ind) = self.ledger.get(id) {
                        let entry = self
                            .movement_by_species
                            .entry(species_key(ind.species.clone()))
                            .or_insert_with(|| json!({"moves":0,"energy_paid":0}));
                        entry["moves"] = json!(entry["moves"].as_u64().unwrap_or(0) + 1);
                        entry["energy_paid"] =
                            json!(entry["energy_paid"].as_u64().unwrap_or(0) + *energy_cost as u64);
                    } else {
                        self.unattributed_actor_events.push(json!({"tick":tick,"event":"EntityMoved","id":id,"energy_cost":energy_cost}));
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
                        self.trait_payment_issues.push(json!({"tick":tick,"id":id,"vision_requested":vision_cost,"vision_paid":vision_paid,"stamina_requested":stamina_cost,"stamina_paid":stamina_paid}));
                    }
                    if let Some(ind) = self.ledger.get(id) {
                        let entry = self.traits_by_species.entry(species_key(ind.species.clone())).or_insert_with(|| json!({"vision_requested":0,"vision_paid":0,"stamina_requested":0,"stamina_paid":0}));
                        for (field, amount) in [
                            ("vision_requested", *vision_cost),
                            ("vision_paid", *vision_paid),
                            ("stamina_requested", *stamina_cost),
                            ("stamina_paid", *stamina_paid),
                        ] {
                            entry[field] =
                                json!(entry[field].as_u64().unwrap_or(0) + amount as u64);
                        }
                    } else {
                        self.unattributed_actor_events
                            .push(json!({"tick":tick,"event":"EntityMetabolized","id":id}));
                    }
                }
                SimulationEvent::ResourceReplenished { kind, amount, .. } => {
                    match kind {
                        ResourceKind::Food => food_renewed += amount,
                        ResourceKind::Water => water_renewed += amount,
                    }
                    add(&mut self.resource_renewals, format!("{:?}", kind), *amount);
                }
                _ => {}
            }
        }
        let now = stock(world);
        for slot in &world.entities {
            if let Some(individual) = self.ledger.get_mut(&slot.id) {
                let life = slot.base.life();
                individual.last_body_observation =
                    json!({"energy":life.energy,"water":life.water,"health":life.health});
                individual.last_body_observation_tick = tick;
            }
        }
        let cells = resource_cells(world);
        let food_residual =
            now.0 as i128 - (previous.0 as i128 + food_renewed as i128 - plant_meals as i128);
        let water_residual =
            now.1 as i128 - (previous.1 as i128 + water_renewed as i128 - drinks as i128);
        self.reconciliation_ticks += 1;
        self.food_residual_total += food_residual;
        self.water_residual_total += water_residual;
        if food_residual < 0 {
            self.food_unexplained_loss_total += food_residual.unsigned_abs();
        }
        if food_residual > 0 {
            self.food_unexplained_gain_total += food_residual as u128;
        }
        if water_residual < 0 {
            self.water_unexplained_loss_total += water_residual.unsigned_abs();
        }
        if water_residual > 0 {
            self.water_unexplained_gain_total += water_residual as u128;
        }
        if food_residual != 0 {
            self.food_residual_nonzero_ticks += 1;
        }
        if water_residual != 0 {
            self.water_residual_nonzero_ticks += 1;
        }
        let reconciliation_row = json!({
            "tick":tick,
            "food": {"start":previous.0,"renewed":food_renewed,"plant_meals":plant_meals,"end":now.0,"unexplained_residual":food_residual},
            "water": {"start":previous.1,"renewed":water_renewed,"drinks":drinks,"end":now.1,"unexplained_residual":water_residual}
        });
        if food_residual != 0 || water_residual != 0 {
            if self.first_reconciliation_anomaly.is_none() {
                self.first_reconciliation_anomaly = Some(reconciliation_row.clone());
            }
            self.last_reconciliation_anomaly = Some(reconciliation_row.clone());
        }
        if self.reconciliation.len() < MAX_DETAIL_ROWS {
            self.reconciliation.push(reconciliation_row);
        }
        let positions: BTreeSet<_> = previous_cells.keys().chain(cells.keys()).copied().collect();
        for pos in positions {
            let before = previous_cells.get(&pos).copied().unwrap_or("empty");
            let after = cells.get(&pos).copied().unwrap_or("empty");
            if before != after && before != "empty" && after != "empty" {
                let row = json!({"tick":tick,"pos":{"x":pos.0,"y":pos.1},"from":before,"to":after,"status":"unexplained_resource_kind_replacement"});
                self.replacement_count += 1;
                if self.first_replacement.is_none() {
                    self.first_replacement = Some(row.clone());
                }
                self.last_replacement = Some(row.clone());
                if self.replacements.len() < MAX_DETAIL_ROWS {
                    self.replacements.push(row);
                }
            }
        }
        let _ = (food_renewed, water_renewed);
        (now, cells)
    }

    pub fn result(&self, world: &World) -> Value {
        let living = world
            .entities
            .iter()
            .filter(|s| {
                s.base.life().is_alive()
                    && matches!(
                        s.phase,
                        curious::entity::phase::EntityPhase::Active
                            | curious::entity::phase::EntityPhase::Sleeping { .. }
                    )
            })
            .map(|s| s.id)
            .collect::<BTreeSet<_>>();
        let generation_two = self.births_by_generation.get(&2).copied().unwrap_or(0);
        let f1_breeders = self
            .bred_ids
            .iter()
            .filter(|id| {
                self.ledger
                    .get(id)
                    .is_some_and(|x| !x.founder && x.generation == Some(1))
            })
            .count();
        let living_bred_descendants = self
            .bred_ids
            .iter()
            .filter(|id| living.contains(id) && self.ledger.get(id).is_some_and(|x| !x.founder))
            .count();
        let individuals: Vec<_> = self.ledger.values().map(|x| json!({
            "id":x.id,"species":format!("{:?}",x.species),"generation":x.generation,"parents":x.parents,
            "founder":x.founder,"died_from":x.death_cause,"bred":x.mated,"births":x.births,"offspring":x.offspring,"snapshot":x.snapshot,
            "last_body_observation":x.last_body_observation,"last_body_observation_tick":x.last_body_observation_tick
        })).collect();
        json!({
            "individual_ledger":individuals,
            "births_by_generation":self.births_by_generation,
            "unknown_generation_births":self.unknown_generation_births,
            "f1_breeders_ever":f1_breeders,
            "f2_births":generation_two,
            "living_bred_descendants_at_end":living_bred_descendants,
            "parent_event_issues":self.parent_event_issues,
            "unattributed_actor_events":self.unattributed_actor_events,
            "trait_payment_issues":self.trait_payment_issues,
            "deaths_by_species_and_cause":self.deaths_by_cause,
            "resource_acquisition_by_species":{"plant_food":self.eaten_plant_by_species,"corpse_food":self.eaten_corpse_by_species,"water":self.drank_by_species},
            "movement_by_species":self.movement_by_species,
            "trait_maintenance_by_species":self.traits_by_species,
            "resource_renewals":self.resource_renewals,
            "plant_meals_total":self.food_meals,"corpse_food_total":self.corpse_meals,"water_drinks_total":self.drinks,
            "resource_reconciliation":{"ticks_checked":self.reconciliation_ticks,"detail_rows_retained":self.reconciliation.len(),"detail_row_limit":MAX_DETAIL_ROWS,"food_nonzero_residual_ticks":self.food_residual_nonzero_ticks,"water_nonzero_residual_ticks":self.water_residual_nonzero_ticks,"food_signed_residual_total":self.food_residual_total,"food_unexplained_loss_total":self.food_unexplained_loss_total,"food_unexplained_gain_total":self.food_unexplained_gain_total,"water_signed_residual_total":self.water_residual_total,"water_unexplained_loss_total":self.water_unexplained_loss_total,"water_unexplained_gain_total":self.water_unexplained_gain_total,"first_anomaly":self.first_reconciliation_anomaly,"last_anomaly":self.last_reconciliation_anomaly,"detail_rows":self.reconciliation},
            "unexplained_resource_kind_replacements":self.replacements,
            "resource_kind_replacement_summary":{"count":self.replacement_count,"detail_rows_retained":self.replacements.len(),"detail_row_limit":MAX_DETAIL_ROWS,"first":self.first_replacement,"last":self.last_replacement},
            "energy_attribution":{"basal_metabolism":"unknown_not_emitted_as_an_event","unexplained_body_energy_change":"unknown_residual_not_inferred"}
        })
    }

    #[allow(dead_code)]
    pub fn generation_cohort(&self, generation: usize) -> Vec<&Individual> {
        self.ledger
            .values()
            .filter(|ind| ind.generation == Some(generation))
            .collect()
    }

    #[allow(dead_code)]
    pub fn reproducing_individuals(&self) -> Vec<&Individual> {
        self.ledger.values().filter(|ind| ind.births > 0).collect()
    }

    #[allow(dead_code)]
    pub fn non_reproducing_survivors<'a>(&'a self, world: &'a World) -> Vec<&'a Individual> {
        let living_ids: BTreeSet<usize> = world
            .entities
            .iter()
            .filter(|s| {
                s.base.life().is_alive()
                    && matches!(
                        s.phase,
                        curious::entity::phase::EntityPhase::Active
                            | curious::entity::phase::EntityPhase::Sleeping { .. }
                    )
            })
            .map(|s| s.id)
            .collect();
        self.ledger
            .values()
            .filter(|ind| living_ids.contains(&ind.id) && ind.births == 0)
            .collect()
    }

    #[allow(dead_code)]
    pub fn lineage_descendants(&self, ancestor_id: usize) -> BTreeSet<usize> {
        let mut descendants = BTreeSet::new();
        let mut queue = Vec::new();
        if let Some(ancestor) = self.ledger.get(&ancestor_id) {
            queue.extend(ancestor.offspring.iter().copied());
        }
        while let Some(child_id) = queue.pop() {
            if descendants.insert(child_id) {
                if let Some(child) = self.ledger.get(&child_id) {
                    queue.extend(child.offspring.iter().copied());
                }
            }
        }
        descendants
    }
}
