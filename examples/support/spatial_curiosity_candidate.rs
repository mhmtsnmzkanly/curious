//! Spatial Known-Map & Curiosity-Driven Exploration Prototype.
//!
//! This module provides coarse-grained per-individual spatial memory and curiosity-driven
//! direction ranking for exploration. It operates strictly within observable boundaries:
//! it never accesses hidden world state and never inherits parent sector maps.

use std::collections::HashMap;

use curious::{
    entity::perception::Perception,
    map::{
        cell::Cell,
        movement::{Direction, Position},
    },
};

pub const SECTOR_SIZE: i32 = 4;
pub const MAX_KNOWN_SECTORS: usize = 16;
pub const SECTOR_MEMORY_TTL_TICKS: usize = 120;

/// Coarse sector coordinates derived from world coordinates via Euclidean division.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SectorCoord {
    pub sx: i32,
    pub sy: i32,
}

impl SectorCoord {
    pub const fn new(sx: i32, sy: i32) -> Self {
        Self { sx, sy }
    }

    pub fn from_world_pos(x: i32, y: i32) -> Self {
        Self {
            sx: x.div_euclid(SECTOR_SIZE),
            sy: y.div_euclid(SECTOR_SIZE),
        }
    }
}

/// Recorded observation record for a single coarse sector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectorEntry {
    pub coord: SectorCoord,
    pub last_visited_tick: usize,
    pub visit_count: u32,
    pub observed_empty_or_blocked: bool,
}

/// Per-individual spatial curiosity memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpatialCuriosityMemory {
    pub enabled: bool,
    pub update_enabled: bool,
    pub sectors: Vec<SectorEntry>,
}

impl Default for SpatialCuriosityMemory {
    fn default() -> Self {
        Self {
            enabled: false,
            update_enabled: true,
            sectors: Vec::new(),
        }
    }
}

impl SpatialCuriosityMemory {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            update_enabled: true,
            sectors: Vec::new(),
        }
    }

    /// Set enabled status. When disabled, queries return None and updates are no-ops.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Set frozen update status. When update_enabled is false, queries still function
    /// based on existing memory, but no new observations or tick updates are written.
    pub fn set_update_enabled(&mut self, update_enabled: bool) {
        self.update_enabled = update_enabled;
    }

    /// Record a visit to a position at a specific tick.
    pub fn record_visit(&mut self, current_tick: usize, pos: (i32, i32), observed_empty_or_blocked: bool) {
        if !self.enabled || !self.update_enabled {
            return;
        }

        let coord = SectorCoord::from_world_pos(pos.0, pos.1);
        if let Some(entry) = self.sectors.iter_mut().find(|e| e.coord == coord) {
            entry.last_visited_tick = current_tick;
            entry.visit_count = entry.visit_count.saturating_add(1);
            entry.observed_empty_or_blocked = observed_empty_or_blocked;
            return;
        }

        // Bounded capacity: evict oldest-visited sector if at capacity.
        if self.sectors.len() >= MAX_KNOWN_SECTORS {
            if let Some(oldest_idx) = self
                .sectors
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| (e.last_visited_tick, e.visit_count, e.coord.sx, e.coord.sy))
                .map(|(idx, _)| idx)
            {
                self.sectors.swap_remove(oldest_idx);
            }
        }

        self.sectors.push(SectorEntry {
            coord,
            last_visited_tick: current_tick,
            visit_count: 1,
            observed_empty_or_blocked,
        });
    }

    /// Returns sector entry if present and not stale.
    pub fn get_sector(&self, coord: SectorCoord) -> Option<&SectorEntry> {
        self.sectors.iter().find(|e| e.coord == coord)
    }

    /// Checks if a sector is considered stale given the current tick.
    pub fn is_stale(&self, entry: &SectorEntry, current_tick: usize) -> bool {
        current_tick.saturating_sub(entry.last_visited_tick) >= SECTOR_MEMORY_TTL_TICKS
    }

    /// Ranks directions and returns the curious explore direction.
    ///
    /// Ranks open directions preferring unvisited or oldest-visited sectors,
    /// tie-breaking by fewest visits, then stable compass order:
    /// Up (0), UpRight (1), Right (2), DownRight (3), Down (4), DownLeft (5), Left (6), UpLeft (7).
    pub fn choose_curious_explore_direction(
        &self,
        origin: (i32, i32),
        current_tick: usize,
        open_directions: &HashMap<Direction, u8>,
        walkable_cells: Option<&[(Position, Cell)]>,
    ) -> Option<Direction> {
        if !self.enabled {
            return None;
        }

        let current_sector = SectorCoord::from_world_pos(origin.0, origin.1);

        // Filter open directions (distance > 0 and not blocked by wall in perceived walkable cells).
        let mut candidates: Vec<Direction> = open_directions
            .iter()
            .filter(|(_, steps)| **steps > 0)
            .map(|(&dir, _)| dir)
            .filter(|&dir| {
                if let Some(cells) = walkable_cells {
                    let (dx, dy) = direction_delta(dir);
                    let target_pos = Position::new(origin.0 as isize + dx as isize, origin.1 as isize + dy as isize);
                    if let Some((_, cell)) = cells.iter().find(|(p, _)| *p == target_pos) {
                        return !matches!(cell, Cell::Wall);
                    }
                }
                true
            })
            .collect();

        if candidates.is_empty() {
            return None;
        }

        // Deterministic sort:
        // Priority bucket:
        // 0: Unvisited (not in memory) or Stale (age >= SECTOR_MEMORY_TTL_TICKS)
        // 1: Known visited (< TTL)
        // Secondary: oldest visited tick (smaller is older)
        // Tertiary: fewest visit counts
        // Quaternary: compass order
        candidates.sort_by_key(|&dir| {
            let (dx, dy) = direction_delta(dir);
            let target_sector = SectorCoord::new(current_sector.sx + dx, current_sector.sy + dy);

            if let Some(entry) = self.get_sector(target_sector) {
                let age = current_tick.saturating_sub(entry.last_visited_tick);
                let is_stale = age >= SECTOR_MEMORY_TTL_TICKS;
                let priority = if is_stale { 0u8 } else { 1u8 };
                let blocked_flag = if entry.observed_empty_or_blocked { 1u8 } else { 0u8 };
                // For stale sectors, older last_visited_tick is still ranked earlier.
                (
                    priority,
                    blocked_flag,
                    entry.last_visited_tick,
                    entry.visit_count,
                    compass_order(dir),
                )
            } else {
                // Completely unvisited: priority 0, never blocked, last_visited 0, visit_count 0.
                (0u8, 0u8, 0usize, 0u32, compass_order(dir))
            }
        });

        candidates.first().copied()
    }

    /// Convenience wrapper taking a full `Perception` struct.
    pub fn choose_from_perception(
        &self,
        origin: (i32, i32),
        current_tick: usize,
        perception: &Perception,
    ) -> Option<Direction> {
        self.choose_curious_explore_direction(
            origin,
            current_tick,
            &perception.directions,
            Some(&perception.cells),
        )
    }
}

/// Direction coordinate offset.
pub const fn direction_delta(dir: Direction) -> (i32, i32) {
    match dir {
        Direction::Up => (0, -1),
        Direction::UpRight => (1, -1),
        Direction::Right => (1, 0),
        Direction::DownRight => (1, 1),
        Direction::Down => (0, 1),
        Direction::DownLeft => (-1, 1),
        Direction::Left => (-1, 0),
        Direction::UpLeft => (-1, -1),
    }
}

/// Direction discriminant in clockwise compass order: U, UR, R, DR, D, DL, L, UL.
pub const fn compass_order(dir: Direction) -> u8 {
    match dir {
        Direction::Up => 0,
        Direction::UpRight => 1,
        Direction::Right => 2,
        Direction::DownRight => 3,
        Direction::Down => 4,
        Direction::DownLeft => 5,
        Direction::Left => 6,
        Direction::UpLeft => 7,
    }
}
