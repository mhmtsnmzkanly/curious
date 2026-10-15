//! Renewable sites are geography, independent of the current population.
use crate::{
    map::{Map, cell::Cell, movement::Position},
    simulation::SimulationEvent,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    Food,
    Water,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSite {
    pub pos: Position,
    pub kind: ResourceKind,
    pub capacity: usize,
    pub replenish: usize,
    pub interval: usize,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Environment {
    pub sites: Vec<ResourceSite>,
}
impl Environment {
    pub fn from_map(map: &Map) -> Self {
        let mut sites = Vec::new();
        for y in map.min_y..=map.max_y {
            for x in map.min_x..=map.max_x {
                let pos = Position::new(x, y);
                let site = match map.cell(pos) {
                    Some(Cell::Food { .. }) => Some(ResourceSite {
                        pos,
                        kind: ResourceKind::Food,
                        capacity: 20,
                        replenish: 1,
                        interval: 2,
                    }),
                    Some(Cell::Water { .. }) => Some(ResourceSite {
                        pos,
                        kind: ResourceKind::Water,
                        capacity: 30,
                        replenish: 4,
                        interval: 2,
                    }),
                    _ => None,
                };
                if let Some(site) = site {
                    sites.push(site);
                }
            }
        }
        Self { sites }
    }
    pub fn update(&self, map: &mut Map, tick: usize, events: &mut Vec<SimulationEvent>) {
        for site in &self.sites {
            if site.interval == 0 || tick % site.interval != 0 {
                continue;
            }
            let current = match (site.kind, map.cell(site.pos)) {
                (ResourceKind::Food, Some(Cell::Food { amount }))
                | (ResourceKind::Water, Some(Cell::Water { amount })) => *amount,
                (_, None | Some(Cell::Empty)) => 0,
                _ => continue, // An intervention may replace a cell with a different resource.
            };
            let amount = site.replenish.min(site.capacity.saturating_sub(current));
            if amount == 0 {
                continue;
            }
            let cell = match site.kind {
                ResourceKind::Food => Cell::Food {
                    amount: current + amount,
                },
                ResourceKind::Water => Cell::Water {
                    amount: current + amount,
                },
            };
            map.set_cell(site.pos, cell);
            events.push(SimulationEvent::ResourceReplenished {
                pos: site.pos,
                kind: site.kind,
                amount,
            });
        }
    }
    pub fn set_site(&mut self, site: ResourceSite) {
        self.sites.retain(|s| s.pos != site.pos);
        self.sites.push(site);
        self.sites.sort_by_key(|s| (s.pos.y, s.pos.x));
    }
}
