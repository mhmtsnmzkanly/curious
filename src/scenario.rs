//! Explicit initial conditions for reproducible ecology experiments.
use crate::{
    Result,
    creatures::from_life as make_entity,
    entity::{controller::EntityController, phase::EntityPhase, species::Species},
    environment::{Environment, ResourceSite},
    map::{Map, cell::Cell, movement::Position},
    world::{EntitySlot, World},
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Founder {
    #[serde(default)]
    pub genome: crate::entity::genetics::Genome,
    #[serde(default)]
    pub life: Option<crate::entity::lifestate::LifeState>,
    pub species: Species,
    pub pos: Position,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub name: String,
    pub bounds: [isize; 4],
    pub founders: Vec<Founder>,
    pub sites: Vec<ResourceSite>,
    pub resources: Vec<(Position, Cell)>,
}
impl Scenario {
    pub fn build(self, seed: u64) -> Result<World> {
        let [a, b, c, d] = self.bounds;
        if a > b
            || c > d
            || a.unsigned_abs() > 1_000_000
            || b.unsigned_abs() > 1_000_000
            || c.unsigned_abs() > 1_000_000
            || d.unsigned_abs() > 1_000_000
            || b - a >= 256
            || d - c >= 256
        {
            return Err("scenario bounds must span at most 256x256 cells".into());
        }
        if self.founders.len() > 10000 {
            return Err("too many founders".into());
        }
        let mut map = Map::new(a, b, c, d);
        let mut occupied = std::collections::HashSet::new();
        let mut entities = Vec::new();
        for (n, f) in self.founders.into_iter().enumerate() {
            if !map.in_bounds(f.pos) || !occupied.insert(f.pos) {
                return Err("invalid or overlapping founder position".into());
            }
            let template: Box<dyn crate::entity::Entity> = match f.species {
                Species::Herbivore => {
                    Box::new(crate::creatures::herbivore::HerbivoreEntity::default())
                }
                Species::Carnivore => {
                    Box::new(crate::creatures::carnivore::CarnivoreEntity::default())
                }
                Species::Omnivore => {
                    Box::new(crate::creatures::omnivore::OmnivoreEntity::default())
                }
            };
            let life = f.life.unwrap_or_else(|| template.life().clone());
            life.validate()?;
            f.genome.validate()?;
            let mut life = life;
            f.genome.express_from(&Default::default(), &mut life);
            entities.push(EntitySlot::new(
                n + 1,
                f.pos,
                EntityPhase::Active,
                EntityController::AI,
                make_entity(f.species, life),
            ));
            entities.last_mut().unwrap().genome = f.genome;
        }
        let mut environment = Environment::default();
        for site in self.sites {
            if !map.in_bounds(site.pos)
                || site.capacity == 0
                || site.capacity > 1_000_000
                || site.replenish > site.capacity
                || site.interval == 0
                || site.interval > 1_000_000
            {
                return Err("invalid resource site".into());
            }
            environment.set_site(site);
        }
        for (pos, cell) in self.resources {
            if !map.in_bounds(pos)
                || matches!(cell,Cell::Food{amount}|Cell::Water{amount} if amount>1_000_000)
            {
                return Err("invalid initial resource".into());
            }
            map.set_cell(pos, cell);
        }
        if entities.iter().any(|e| !map.is_walkable(e.pos)) {
            return Err("founder position is blocked".into());
        }
        let next_entity_id = entities.len() + 1;
        Ok(World {
            genetics: Default::default(),
            observation: Default::default(),
            last_decisions: Vec::new(),
            map,
            environment,
            entities,
            next_entity_id,
            rng: crate::random::Random::new(seed),
            tick_counter: 0,
            logger: crate::logger::Logger::disabled(),
            command_inputs: Default::default(),
            last_trace: Vec::new(),
            last_intents: Vec::new(),
        })
    }
}
