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

pub const MAX_ENVIRONMENT_SIGNALS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SignalKind {
    Footstep,
    Scent,
    Sound,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalTrace {
    pub origin: Position,
    pub kind: SignalKind,
    pub intensity: u16,
    pub radius: u8,
    pub decay_per_tick: u16,
    pub emitted_tick: usize,
}

impl SignalTrace {
    pub fn validate(&self) -> crate::Result<()> {
        if self.origin.x.unsigned_abs() > 1_000_000
            || self.origin.y.unsigned_abs() > 1_000_000
            || self.intensity == 0
            || self.radius == 0
            || self.radius > 64
            || self.decay_per_tick == 0
        {
            return Err("invalid signal trace".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PerceivedSignal {
    pub kind: SignalKind,
    pub direction_sector: u8,
    pub distance: u8,
    pub local_intensity: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DayNightCycle {
    pub period: usize,
    pub day_ticks: usize,
    pub night_vision_penalty: usize,
}

impl DayNightCycle {
    pub fn validate(&self) -> crate::Result<()> {
        if self.period == 0
            || self.period > 1_000_000
            || self.day_ticks > self.period
            || self.night_vision_penalty > 32
        {
            return Err("invalid day-night cycle".into());
        }
        Ok(())
    }

    pub fn is_day(&self, tick: usize) -> bool {
        if self.period == 0 {
            return true;
        }
        tick.rem_euclid(self.period) < self.day_ticks
    }

    pub fn effective_vision_range(&self, tick: usize, base_vision: usize) -> usize {
        if self.is_day(tick) {
            base_vision
        } else {
            base_vision.saturating_sub(self.night_vision_penalty).max(1)
        }
    }
}

pub fn calculate_direction_sector(from: Position, to: Position) -> u8 {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    match (dx.signum(), dy.signum()) {
        (0, -1) => 0,
        (1, -1) => 1,
        (1, 0) => 2,
        (1, 1) => 3,
        (0, 1) => 4,
        (-1, 1) => 5,
        (-1, 0) => 6,
        (-1, -1) => 7,
        _ => 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeasonCycle {
    pub period: usize,
    pub winter_bmr_surcharge: u16,
    pub winter_plant_regen_divisor: usize,
}

impl SeasonCycle {
    pub fn validate(&self) -> crate::Result<()> {
        if self.period == 0 || self.period > 1_000_000 {
            return Err("invalid season cycle period".into());
        }
        if self.winter_plant_regen_divisor == 0 {
            return Err("winter plant regen divisor must be > 0".into());
        }
        if self.winter_bmr_surcharge > 100 {
            return Err("winter bmr surcharge exceeds limit".into());
        }
        Ok(())
    }

    pub fn current_season(&self, tick: usize) -> Season {
        if self.period == 0 {
            return Season::Spring;
        }
        let phase = tick.rem_euclid(self.period);
        let quarter = phase * 4 / self.period;
        match quarter {
            0 => Season::Spring,
            1 => Season::Summer,
            2 => Season::Autumn,
            _ => Season::Winter,
        }
    }
}

pub const MAX_BURROWS: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BurrowSite {
    pub pos: Position,
    pub created_tick: usize,
    pub ttl: usize,
}

impl BurrowSite {
    pub fn validate(&self) -> crate::Result<()> {
        if self.pos.x.unsigned_abs() > 1_000_000 || self.pos.y.unsigned_abs() > 1_000_000 {
            return Err("invalid burrow position".into());
        }
        if self.ttl == 0 || self.ttl > 1_000_000 {
            return Err("invalid burrow ttl".into());
        }
        Ok(())
    }
}

fn is_false(v: &bool) -> bool {
    !*v
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Environment {
    pub sites: Vec<ResourceSite>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub day_night: Option<DayNightCycle>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub signals_enabled: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signals: Vec<SignalTrace>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub season_cycle: Option<SeasonCycle>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub burrows: Vec<BurrowSite>,
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
        Self {
            sites,
            day_night: None,
            signals_enabled: false,
            signals: Vec::new(),
            season_cycle: None,
            burrows: Vec::new(),
        }
    }
    pub fn update(&mut self, map: &mut Map, tick: usize, events: &mut Vec<SimulationEvent>) {
        // Revert physical map cells for expired burrows
        for b in &self.burrows {
            if tick.saturating_sub(b.created_tick) >= b.ttl {
                if map.cell(b.pos) == Some(&Cell::Burrow) {
                    map.set_cell(b.pos, Cell::Empty);
                }
            }
        }
        self.burrows.retain(|b| tick.saturating_sub(b.created_tick) < b.ttl);

        if self.signals_enabled {
            self.signals.retain_mut(|signal| {
                signal.intensity = signal.intensity.saturating_sub(signal.decay_per_tick);
                signal.intensity > 0
            });
        }
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
            let mut replenish = site.replenish;
            if site.kind == ResourceKind::Food {
                if let Some(cycle) = self.season_cycle {
                    if cycle.current_season(tick) == Season::Winter && cycle.winter_plant_regen_divisor > 1 {
                        replenish = (site.replenish / cycle.winter_plant_regen_divisor)
                            .max(if site.replenish > 0 { 1 } else { 0 });
                    }
                }
            }
            let amount = replenish.min(site.capacity.saturating_sub(current));
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
    pub fn emit_signal(&mut self, trace: SignalTrace) {
        if !self.signals_enabled || trace.intensity == 0 || trace.radius == 0 {
            return;
        }
        if trace.validate().is_err() {
            return;
        }
        if self.signals.len() >= MAX_ENVIRONMENT_SIGNALS {
            if let Some((idx, _)) = self
                .signals
                .iter()
                .enumerate()
                .min_by_key(|(_, s)| (s.emitted_tick, s.intensity, s.origin.y, s.origin.x))
            {
                self.signals.remove(idx);
            }
        }
        self.signals.push(trace);
    }
    pub fn perceive_signals_at(
        &self,
        listener_pos: Position,
        current_tick: usize,
    ) -> Vec<PerceivedSignal> {
        if !self.signals_enabled {
            return Vec::new();
        }
        let mut perceived = Vec::new();
        for signal in &self.signals {
            if signal.emitted_tick > current_tick {
                continue;
            }
            let dx = (signal.origin.x - listener_pos.x).unsigned_abs();
            let dy = (signal.origin.y - listener_pos.y).unsigned_abs();
            let chebyshev_dist = dx.max(dy);
            if chebyshev_dist > signal.radius as usize {
                continue;
            }
            let distance = chebyshev_dist as u8;
            let attenuation = (distance as u16).saturating_mul(signal.decay_per_tick);
            let local_intensity = signal.intensity.saturating_sub(attenuation);
            if local_intensity == 0 {
                continue;
            }
            let direction_sector = calculate_direction_sector(listener_pos, signal.origin);
            perceived.push(PerceivedSignal {
                kind: signal.kind,
                direction_sector,
                distance,
                local_intensity,
            });
        }
        perceived.sort_by_key(|p| (p.distance, p.direction_sector, p.kind as u8, std::cmp::Reverse(p.local_intensity)));
        perceived
    }
    pub fn dig_burrow(&mut self, pos: Position, tick: usize, ttl: usize) -> bool {
        if ttl == 0 || pos.x.unsigned_abs() > 1_000_000 || pos.y.unsigned_abs() > 1_000_000 {
            return false;
        }
        self.burrows.retain(|b| tick.saturating_sub(b.created_tick) < b.ttl);
        if let Some(existing) = self.burrows.iter_mut().find(|b| b.pos == pos) {
            existing.created_tick = tick;
            existing.ttl = ttl;
            return true;
        }
        if self.burrows.len() >= MAX_BURROWS {
            return false;
        }
        self.burrows.push(BurrowSite {
            pos,
            created_tick: tick,
            ttl,
        });
        true
    }

    pub fn dig_burrow_on_map(&mut self, map: &mut Map, pos: Position, tick: usize, ttl: usize) -> bool {
        if ttl == 0 || pos.x < map.min_x || pos.x > map.max_x || pos.y < map.min_y || pos.y > map.max_y {
            return false;
        }
        match map.cell(pos) {
            Some(&Cell::Empty) => {
                if self.dig_burrow(pos, tick, ttl) {
                    map.set_cell(pos, Cell::Burrow);
                    true
                } else {
                    false
                }
            }
            Some(&Cell::Burrow) => {
                if let Some(existing) = self
                    .burrows
                    .iter_mut()
                    .find(|b| b.pos == pos && tick.saturating_sub(b.created_tick) < b.ttl)
                {
                    existing.created_tick = tick;
                    existing.ttl = ttl;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn has_active_burrow(&self, pos: Position, tick: usize) -> bool {
        self.burrows
            .iter()
            .any(|b| b.pos == pos && tick.saturating_sub(b.created_tick) < b.ttl)
    }

    pub fn winter_bmr_surcharge(&self, pos: Position, tick: usize) -> u16 {
        let Some(cycle) = self.season_cycle else {
            return 0;
        };
        if cycle.current_season(tick) != Season::Winter {
            return 0;
        }
        if self.has_active_burrow(pos, tick) {
            0
        } else {
            cycle.winter_bmr_surcharge
        }
    }

    pub fn validate(&self) -> crate::Result<()> {
        for site in &self.sites {
            site.validate()?;
        }
        if let Some(cycle) = self.day_night {
            cycle.validate()?;
        }
        if let Some(cycle) = self.season_cycle {
            cycle.validate()?;
        }
        if self.burrows.len() > MAX_BURROWS {
            return Err("too many burrows".into());
        }
        for burrow in &self.burrows {
            burrow.validate()?;
        }
        if self.signals.len() > MAX_ENVIRONMENT_SIGNALS {
            return Err("too many environment signals".into());
        }
        for signal in &self.signals {
            signal.validate()?;
        }
        Ok(())
    }
    pub fn set_site(&mut self, site: ResourceSite) {
        self.sites.retain(|s| s.pos != site.pos);
        self.sites.push(site);
        self.sites.sort_by_key(|s| (s.pos.y, s.pos.x));
    }
}
