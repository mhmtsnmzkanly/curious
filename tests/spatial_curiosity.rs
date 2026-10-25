#[path = "../examples/support/spatial_curiosity_candidate.rs"]
mod spatial_curiosity_candidate;

use std::collections::HashMap;

use curious::map::{
    cell::Cell,
    movement::{Direction, Position},
};
use spatial_curiosity_candidate::*;

#[test]
fn test_sector_coord_euclidean_division_boundaries() {
    assert_eq!(SectorCoord::from_world_pos(0, 0), SectorCoord::new(0, 0));
    assert_eq!(SectorCoord::from_world_pos(3, 3), SectorCoord::new(0, 0));
    assert_eq!(SectorCoord::from_world_pos(4, 0), SectorCoord::new(1, 0));
    assert_eq!(SectorCoord::from_world_pos(7, 4), SectorCoord::new(1, 1));

    // Negative coordinates: Euclidean division div_euclid(-1, 4) == -1
    assert_eq!(SectorCoord::from_world_pos(-1, -1), SectorCoord::new(-1, -1));
    assert_eq!(SectorCoord::from_world_pos(-4, -4), SectorCoord::new(-1, -1));
    assert_eq!(SectorCoord::from_world_pos(-5, -5), SectorCoord::new(-2, -2));
    assert_eq!(SectorCoord::from_world_pos(-8, 0), SectorCoord::new(-2, 0));
}

#[test]
fn test_bounded_memory_capacity_and_oldest_eviction() {
    let mut mem = SpatialCuriosityMemory::new(true);

    // Insert 16 unique sectors at successive ticks
    for i in 0..MAX_KNOWN_SECTORS {
        mem.record_visit(i * 10, (i as i32 * 4, 0), false);
    }
    assert_eq!(mem.sectors.len(), MAX_KNOWN_SECTORS);

    // Oldest sector is (0, 0) visited at tick 0.
    assert!(mem.get_sector(SectorCoord::new(0, 0)).is_some());

    // Insert 17th sector at tick 200
    mem.record_visit(200, (100, 100), false);
    assert_eq!(mem.sectors.len(), MAX_KNOWN_SECTORS);

    // The oldest sector (0, 0) visited at tick 0 must have been evicted
    assert!(mem.get_sector(SectorCoord::new(0, 0)).is_none());
    // The new sector at (25, 25) must be present
    assert!(mem.get_sector(SectorCoord::new(25, 25)).is_some());
}

#[test]
fn test_staleness_ttl_re_explores_after_120_ticks() {
    let mut mem = SpatialCuriosityMemory::new(true);

    // Sector (1, 0) visited at tick 10
    mem.record_visit(10, (4, 0), false);
    // Sector (0, 1) visited at tick 110
    mem.record_visit(110, (0, 4), false);

    // At tick 120: sector (1, 0) age is 110 (< 120), so not stale yet
    let entry_east = mem.get_sector(SectorCoord::new(1, 0)).unwrap();
    assert!(!mem.is_stale(entry_east, 120));

    // At tick 135: sector (1, 0) age is 125 (>= 120), becoming stale/unexplored again!
    assert!(mem.is_stale(entry_east, 135));

    // Sector (0, 1) age at tick 135 is 25 (< 120), still recently visited
    let entry_south = mem.get_sector(SectorCoord::new(0, 1)).unwrap();
    assert!(!mem.is_stale(entry_south, 135));

    // Origin at (0, 0) with open directions Right (leads to stale sector (1, 0))
    // and Down (leads to recently visited sector (0, 1))
    let mut open_dirs = HashMap::new();
    open_dirs.insert(Direction::Right, 2);
    open_dirs.insert(Direction::Down, 2);

    let chosen = mem.choose_curious_explore_direction((0, 0), 135, &open_dirs, None);
    // Stale sector (1, 0) has priority 0 (unexplored again) over recently visited sector (0, 1)
    assert_eq!(chosen, Some(Direction::Right));
}

#[test]
fn test_choose_curious_explore_direction_ranking_and_tie_breaking() {
    let mut mem = SpatialCuriosityMemory::new(true);

    // Sectors visited at different ticks
    mem.record_visit(50, (0, -4), false); // Up -> Sector(0, -1) visited tick 50
    mem.record_visit(20, (4, 0), false);  // Right -> Sector(1, 0) visited tick 20 (older)

    let mut open_dirs = HashMap::new();
    open_dirs.insert(Direction::Up, 3);
    open_dirs.insert(Direction::Right, 3);
    open_dirs.insert(Direction::Down, 3); // Down -> Sector(0, 1) UNVISITED

    // 1. Unvisited sector (Down) must beat visited sectors (Up, Right)
    let chosen = mem.choose_curious_explore_direction((0, 0), 60, &open_dirs, None);
    assert_eq!(chosen, Some(Direction::Down));

    // 2. Remove Down. Between Up (tick 50) and Right (tick 20):
    // Oldest-visited (Right, tick 20) must be preferred over newer-visited (Up, tick 50)
    open_dirs.remove(&Direction::Down);
    let chosen2 = mem.choose_curious_explore_direction((0, 0), 60, &open_dirs, None);
    assert_eq!(chosen2, Some(Direction::Right));

    // 3. Compass tie-breaking when both unvisited:
    let mut tie_dirs = HashMap::new();
    tie_dirs.insert(Direction::Left, 2);  // compass order 6
    tie_dirs.insert(Direction::Up, 2);    // compass order 0
    let fresh_mem = SpatialCuriosityMemory::new(true);
    let chosen_tie = fresh_mem.choose_curious_explore_direction((0, 0), 10, &tie_dirs, None);
    assert_eq!(chosen_tie, Some(Direction::Up)); // Up wins by compass tie-breaking

    // 4. Perceived blocked cell (Wall) is avoided:
    let cells = vec![(Position::new(0, -1), Cell::Wall)];
    let chosen_unblocked = fresh_mem.choose_curious_explore_direction((0, 0), 10, &tie_dirs, Some(&cells));
    assert_eq!(chosen_unblocked, Some(Direction::Left));
}

#[test]
fn test_disabled_and_frozen_modes() {
    let mut mem = SpatialCuriosityMemory::new(false); // disabled
    let mut open_dirs = HashMap::new();
    open_dirs.insert(Direction::Up, 2);

    // Disabled mode is strict no-op: returns None and records no updates
    assert_eq!(mem.choose_curious_explore_direction((0, 0), 10, &open_dirs, None), None);
    mem.record_visit(10, (0, -4), false);
    assert_eq!(mem.sectors.len(), 0);

    // Enable memory
    mem.set_enabled(true);
    mem.record_visit(10, (0, -4), false);
    assert_eq!(mem.sectors.len(), 1);

    // Frozen mode: update_enabled = false
    mem.set_update_enabled(false);
    // Queries still work
    let chosen = mem.choose_curious_explore_direction((0, 0), 20, &open_dirs, None);
    assert_eq!(chosen, Some(Direction::Up));

    // But record_visit does not mutate state in frozen mode
    mem.record_visit(30, (4, 0), false);
    assert_eq!(mem.sectors.len(), 1); // New sector (1, 0) was not added
}

#[test]
fn test_zero_inheritance_and_perception_boundary() {
    let mut parent_mem = SpatialCuriosityMemory::new(true);
    for i in 0..5 {
        parent_mem.record_visit(i * 10, (i as i32 * 4, 0), false);
    }
    assert_eq!(parent_mem.sectors.len(), 5);

    // Child must start with clean/default state (zero inheritance)
    let child_mem = SpatialCuriosityMemory::default();
    assert_eq!(child_mem.sectors.len(), 0);
    assert!(!child_mem.enabled); // default off

    let activated_child = SpatialCuriosityMemory::new(true);
    assert_eq!(activated_child.sectors.len(), 0);

    // All sectors are unvisited to child, regardless of parent's knowledge
    let mut open_dirs = HashMap::new();
    open_dirs.insert(Direction::Right, 2); // Parent visited (1, 0), child has not
    open_dirs.insert(Direction::Down, 2);
    let chosen = activated_child.choose_curious_explore_direction((0, 0), 50, &open_dirs, None);
    // Tie-break between unvisited sectors (1, 0) [Right, order 2] and (0, 1) [Down, order 4]
    // Right (2) < Down (4) by compass order
    assert_eq!(chosen, Some(Direction::Right));

    // Also verify choose_from_perception wrapper
    let mut perception = curious::entity::perception::Perception::empty();
    perception.directions = open_dirs;
    let chosen_p = activated_child.choose_from_perception((0, 0), 50, &perception);
    assert_eq!(chosen_p, Some(Direction::Right));
}
