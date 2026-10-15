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
            version: 8,
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
    pub fn restore(self) -> Result<World> {
        if !(1..=8).contains(&self.version) {
            return Err("unsupported checkpoint version".into());
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
