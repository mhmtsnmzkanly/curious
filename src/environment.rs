//! Renewable sites are geography, independent of the current population.
use crate::{
    map::{cell::Cell, movement::Position, Map},
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
    /// Optional, deterministic geographic renewal window. Missing on older
    /// checkpoints and disabled for map-derived sites.
    #[serde(default)]
    pub schedule: Option<ResourceSchedule>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceSchedule {
    pub period: usize,
    pub active_ticks: usize,
    #[serde(default = "default_geographic_phase_step")]
    pub geographic_phase_step: usize,
    #[serde(default)]
    pub phase_offset: isize,
}

impl ResourceSchedule {
    pub fn validate(&self) -> crate::Result<()> {
        if self.period == 0
            || self.period > 1_000_000
            || self.active_ticks > self.period
            || self.geographic_phase_step > 1_000_000
            || self.phase_offset.unsigned_abs() > 1_000_000
        {
            return Err("invalid resource availability schedule".into());
        }
        Ok(())
    }

    /// Availability depends only on tick and site geography. i128 keeps the
    /// bounded coordinates and settings safe before the modulo operation.
    pub fn is_available(&self, tick: usize, pos: Position) -> bool {
        let period = self.period as i128;
        if period == 0 {
            return false;
        }
        let spatial = (pos.x as i128 + 3 * pos.y as i128) * self.geographic_phase_step as i128
            + self.phase_offset as i128;
        let phase = (tick as i128 + spatial).rem_euclid(period);
        phase < self.active_ticks as i128
    }
}

fn default_geographic_phase_step() -> usize {
    1
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Environment {
    pub sites: Vec<ResourceSite>,
}
impl ResourceSite {
    pub fn validate(&self) -> crate::Result<()> {
        if self.capacity > 1_000_000
            || self.replenish > self.capacity
            || self.interval == 0
            || self.interval > 1_000_000
        {
            return Err("invalid resource site".into());
        }
        if let Some(schedule) = self.schedule {
            schedule.validate()?;
        }
        Ok(())
    }
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
                        schedule: None,
                    }),
                    Some(Cell::Water { .. }) => Some(ResourceSite {
                        pos,
                        kind: ResourceKind::Water,
                        capacity: 30,
                        replenish: 4,
                        interval: 2,
                        schedule: None,
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
            if site
                .schedule
                .is_some_and(|schedule| !schedule.is_available(tick, site.pos))
            {
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
