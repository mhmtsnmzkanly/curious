//! Versioned engine state, independent of SQLite and HTTP.
use crate::{
    Result,
    creatures::from_life as make_entity,
    entity::{
        controller::EntityController, intent::Intent, lifestate::LifeState, phase::EntityPhase,
        species::Species,
    },
    map::{Map, cell::Cell, movement::Position},
    world::{EntitySlot, World},
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Individual {
    #[serde(default)]
    pub genome: crate::entity::genetics::Genome,
    #[serde(default)]
    pub learner: crate::entity::learning::Learner,
    #[serde(default)]
    pub hunt_learner: crate::entity::hunt_learning::HuntLearner,
    #[serde(default)]
    pub memory: crate::entity::memory::Memory,
    #[serde(default)]
    pub generation: usize,
    #[serde(default)]
    pub parents: Vec<usize>,
    #[serde(default)]
    pub corpse_energy: usize,
    pub id: usize,
    pub pos: Position,
    pub phase: EntityPhase,
    pub controller: EntityController,
    pub species: Species,
    pub life: LifeState,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    #[serde(default)]
    pub policy: crate::policy::PolicySettings,
    #[serde(default)]
    pub genetics: crate::entity::genetics::Genetics,
    #[serde(default)]
    pub observation: crate::entity::decision::ObservationSettings,
    #[serde(default)]
    pub environment: crate::environment::Environment,
    pub version: u32,
    pub tick: usize,
    pub rng: u64,
    pub next_entity_id: usize,
    pub bounds: [isize; 4],
    pub entities: Vec<Individual>,
    pub resources: Vec<(Position, Cell)>,
    pub commands: Vec<(usize, Intent)>,
}
impl State {
    pub fn capture(world: &World) -> Self {
        let mut resources = Vec::new();
        for y in world.map.min_y..=world.map.max_y {
            for x in world.map.min_x..=world.map.max_x {
                let p = Position::new(x, y);
                if let Some(
                    cell @ (Cell::Food { .. } | Cell::Water { .. } | Cell::Wall | Cell::Burrow),
                ) = world.map.cell(p)
                {
                    resources.push((p, cell.clone()));
                }
            }
        }
        let mut commands: Vec<_> = world
            .command_inputs
            .iter()
            .map(|(id, i)| (*id, i.clone()))
            .collect();
        commands.sort_by_key(|(id, _)| *id);
        Self {
            policy: world.policy.clone(),
            version: 21,
            genetics: world.genetics.clone(),
            observation: world.observation.clone(),
            environment: world.environment.clone(),
            tick: world.tick_counter,
            rng: world.rng.state(),
            next_entity_id: world.next_entity_id,
            bounds: [
                world.map.min_x,
                world.map.max_x,
                world.map.min_y,
                world.map.max_y,
            ],
            resources,
            commands,
            entities: world
                .entities
                .iter()
                .map(|s| Individual {
                    genome: s.genome.clone(),
                    learner: s.learner.clone(),
                    hunt_learner: s.hunt_learner.clone(),
                    memory: s.memory.clone(),
                    generation: s.generation,
                    parents: s.parents.clone(),
                    corpse_energy: s.corpse_energy,
                    id: s.id,
                    pos: s.pos,
                    phase: s.phase,
                    controller: s.controller.clone(),
                    species: s.base.species(),
                    life: s.base.life().clone(),
                })
                .collect(),
        }
    }
    pub fn restore(mut self) -> Result<World> {
        if !(1..=21).contains(&self.version) {
            return Err("unsupported checkpoint version".into());
        }
        if self.version >= 15 {
            self.genetics.validate()?;
            self.environment.validate()?;
            let [min_x, max_x, min_y, max_y] = self.bounds;
            let mut sites = std::collections::HashSet::new();
            for site in &self.environment.sites {
                site.validate()?;
                if site.pos.x < min_x
                    || site.pos.x > max_x
                    || site.pos.y < min_y
                    || site.pos.y > max_y
                    || !sites.insert(site.pos)
                {
                    return Err("invalid or duplicate resource site in checkpoint".into());
                }
            }
        }
        if self.version >= 19
            && (self.environment.signals.iter().any(|s| s.emitted_tick > self.tick)
                || self.environment.burrows.iter().any(|b| b.created_tick > self.tick))
        {
            return Err("future environment timestamp in checkpoint".into());
        }
        if self.version < 12 {
            self.policy.mate_rendezvous = false;
            for individual in &mut self.entities {
                individual.memory.social = Default::default();
            }
        }
        if self.version < 13 {
            for individual in &mut self.entities {
                individual.hunt_learner = Default::default();
                individual.learner.update_enabled = true;
            }
        }
        if self.version < 14 {
            for individual in &mut self.entities {
                individual.hunt_learner.risk_enabled = false;
                individual.hunt_learner.risk_values.clear();
                if let Some(episode) = &mut individual.hunt_learner.episode {
                    episode.prey_defense_damage = 0;
                }
                if let Some(result) = &mut individual.hunt_learner.last_episode {
                    result.attempt.prey_defense_damage = 0;
                }
                for result in &mut individual.hunt_learner.pending_results {
                    result.attempt.prey_defense_damage = 0;
                }
            }
        }
        if self.version < 15 {
            self.genetics.extra_traits_enabled = false;
            for site in &mut self.environment.sites {
                site.schedule = None;
            }
            for individual in &mut self.entities {
                individual.genome.vision = 0;
                individual.genome.stamina = 0;
                individual.life.vision_metabolic_cost = 0;
                individual.life.stamina_metabolic_cost = 0;
            }
        }
        if self.version < 16 {
            for individual in &mut self.entities {
                individual.memory.refuge_outcomes_enabled = false;
                individual.memory.refuge_outcomes.clear();
                individual.memory.refuge_episode = None;
                individual.memory.pending_refuge_closures.clear();
            }
        }
        if self.version < 17 {
            self.policy.water_residual_enabled = false;
            for individual in &mut self.entities {
                individual.learner.residuals.clear();
                if let Some(attempt) = &mut individual.learner.episode {
                    attempt.choice_baseline = None;
                    attempt.outcome_overflowed = false;
                    attempt.candidate_censored = false;
                }
                if let Some(result) = &mut individual.learner.last_episode {
                    result.attempt.choice_baseline = None;
                    result.attempt.outcome_overflowed = false;
                    result.attempt.candidate_censored = false;
                }
            }
        }
        if self.version < 18 {
            self.policy.injury_capacity_penalty = false;
            for individual in &mut self.entities {
                individual.memory.refuge_generalization_enabled = false;
                individual.memory.refuge_generalization_update_enabled = true;
                individual.memory.refuge_generalized_outcomes.clear();
                if let Some(episode) = &mut individual.memory.refuge_episode {
                    episode.learning_mode = crate::entity::memory::RefugeLearningMode::LegacyExact;
                    episode.generalized_key = None;
                }
                for closure in &mut individual.memory.pending_refuge_closures {
                    closure.episode.learning_mode =
                        crate::entity::memory::RefugeLearningMode::LegacyExact;
                    closure.episode.generalized_key = None;
                }
            }
        }
        if self.version < 19 {
            self.policy.juvenile_care_enabled = false;
            self.policy.ambush_predation_enabled = false;
            self.policy.local_coordination_enabled = false;
            self.environment.day_night = None;
            self.environment.signals_enabled = false;
            self.environment.signals.clear();
            self.environment.season_cycle = None;
            self.environment.burrows.clear();
            for individual in &mut self.entities {
                individual.learner.context_ttl_enabled = false;
                individual.learner.context_entries.clear();
                individual.memory.spatial_curiosity_enabled = false;
                individual.memory.known_sectors.clear();
                individual.memory.first_interval_refuge_enabled = false;
                individual.memory.first_interval_outcomes.clear();
            }
        }
        if self.version < 20 {
            self.policy.food_search_reserve_enabled = false;
        }
        if self.version < 21 {
            self.policy.water_meal_priority_enabled = false;
        }
        for individual in &self.entities {
            if self.version >= 15 {
                individual.genome.validate()?;
            }
            individual.memory.validate()?;
            individual.learner.validate()?;
            individual.hunt_learner.validate()?;
            if self.version >= 18
                && (individual
                    .memory
                    .refuge_generalized_outcomes
                    .iter()
                    .any(|row| {
                        row.last_completed_tick > self.tick
                            || row.key.subject_species != individual.species
                    })
                    || individual
                        .memory
                        .refuge_episode
                        .as_ref()
                        .is_some_and(|episode| {
                            episode.learning_mode
                                == crate::entity::memory::RefugeLearningMode::Generalized
                                && (episode.started_tick > self.tick
                                    || episode.generalized_key.as_ref().is_some_and(|key| {
                                        key.subject_species != individual.species
                                    }))
                        })
                    || individual
                        .memory
                        .pending_refuge_closures
                        .iter()
                        .any(|closure| {
                            closure.episode.learning_mode
                                == crate::entity::memory::RefugeLearningMode::Generalized
                                && (closure.episode.started_tick > self.tick
                                    || closure.ended_tick > self.tick
                                    || closure.ended_tick < closure.episode.started_tick
                                    || closure.episode.generalized_key.as_ref().is_some_and(
                                        |key| key.subject_species != individual.species,
                                    ))
                        }))
            {
                return Err("future generalized refuge timestamp in checkpoint".into());
            }
            if self.version >= 19
                && (individual.learner.context_entries.iter().any(|e| e.last_seen_tick > self.tick)
                    || individual.memory.known_sectors.iter().any(|s| s.last_visited_tick > self.tick)
                    || individual.memory.first_interval_outcomes.iter().any(|r| r.episodes.iter().any(|&(t, _)| t > self.tick)))
            {
                return Err("future V19 learning or memory timestamp in checkpoint".into());
            }
        }
        let [a, b, c, d] = self.bounds;
        let mut map = Map::new(a, b, c, d);
        for (p, cell) in self.resources {
            map.set_cell(p, cell);
        }
        let entities = self
            .entities
            .into_iter()
            .map(|s| {
                let mut slot = EntitySlot::new(
                    s.id,
                    s.pos,
                    s.phase,
                    s.controller,
                    make_entity(s.species, s.life),
                );
                slot.genome = s.genome;
                slot.learner = s.learner;
                slot.hunt_learner = s.hunt_learner;
                slot.memory = s.memory;
                slot.generation = s.generation;
                slot.parents = s.parents;
                slot.corpse_energy = s.corpse_energy;
                if self.version == 1 && matches!(slot.phase, EntityPhase::Corpse { .. }) {
                    slot.corpse_energy = (slot.base.life().max_health
                        / crate::rules::biology::CORPSE_FOOD_DIVISOR)
                        .max(crate::rules::biology::MIN_FOOD_FROM_CORPSE);
                }
                slot
            })
            .collect();
        let environment = if self.version == 1 {
            crate::environment::Environment::from_map(&map)
        } else {
            self.environment
        };
        Ok(World {
            policy: self.policy,
            genetics: self.genetics,
            observation: self.observation,
            last_decisions: Vec::new(),
            environment,
            rng: crate::random::Random::new(self.rng),
            map,
            entities,
            tick_counter: self.tick,
            logger: crate::logger::Logger::disabled(),
            command_inputs: self.commands.into_iter().collect(),
            last_trace: Vec::new(),
            last_intents: Vec::new(),
            next_entity_id: self.next_entity_id,
        })
    }
}
