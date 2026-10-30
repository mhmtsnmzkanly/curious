//! Pure deterministic food patch transform and geometry summaries.
use curious::{
    environment::ResourceKind,
    map::{cell::Cell, movement::Position},
    scenario::Scenario,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

const PATCH_COLUMNS: usize = 4;
const PATCH_ROWS: usize = 2;
const SITES_PER_PATCH: usize = 16;
const PATCH_SITES: usize = PATCH_COLUMNS * PATCH_ROWS * SITES_PER_PATCH;

#[derive(Clone, Debug, Serialize)]
pub struct PatchReport {
    pub anchors: Vec<(isize, isize)>,
    pub patch_positions: Vec<Vec<(isize, isize)>>,
    pub initial_food: usize,
    pub food_site_count: usize,
    pub potential_capacity: usize,
    pub nominal_replenish_per_tick: usize,
    pub intervals: BTreeMap<usize, usize>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FounderGeometry {
    pub genotype: String,
    pub founders: usize,
    pub nearest_food_distance_counts: BTreeMap<usize, usize>,
    pub within_base_range_6: usize,
    pub at_distance_7: usize,
}

fn tuple(pos: Position) -> (isize, isize) {
    (pos.x, pos.y)
}

fn anchors(bounds: [isize; 4]) -> Result<Vec<Position>, String> {
    let [min_x, max_x, min_y, max_y] = bounds;
    if min_x > max_x || min_y > max_y {
        return Err("invalid scenario bounds".into());
    }
    let width = (max_x - min_x + 1) as i128;
    let height = (max_y - min_y + 1) as i128;
    let mut result = Vec::with_capacity(PATCH_COLUMNS * PATCH_ROWS);
    for row in 0..PATCH_ROWS {
        for column in 0..PATCH_COLUMNS {
            let x_offset = ((2 * column + 1) as i128 * width) / (2 * PATCH_COLUMNS) as i128;
            let y_offset = ((2 * row + 1) as i128 * height) / (2 * PATCH_ROWS) as i128;
            result.push(Position::new(
                min_x + x_offset as isize,
                min_y + y_offset as isize,
            ));
        }
    }
    Ok(result)
}

fn manhattan(a: Position, b: Position) -> usize {
    a.x.abs_diff(b.x).saturating_add(a.y.abs_diff(b.y))
}

fn stock_and_sites(
    scenario: &Scenario,
) -> Result<(BTreeMap<(isize, isize), usize>, usize, usize), String> {
    let mut stocks = BTreeMap::new();
    for (pos, cell) in &scenario.resources {
        if let Cell::Food { amount } = cell {
            if stocks.insert(tuple(*pos), *amount).is_some() {
                return Err(format!("duplicate resource cell at {pos:?}"));
            }
        }
    }
    let mut capacities = 0usize;
    let mut replenish = 0usize;
    let mut food_site_positions = BTreeSet::new();
    for site in &scenario.sites {
        if site.kind == ResourceKind::Food {
            if !food_site_positions.insert(tuple(site.pos)) {
                return Err(format!("duplicate Food site at {:?}", site.pos));
            }
            if !stocks.contains_key(&tuple(site.pos)) {
                return Err(format!("Food site has no Food cell at {:?}", site.pos));
            }
            capacities = capacities
                .checked_add(site.capacity)
                .ok_or("Food capacity total overflow")?;
            replenish = replenish
                .checked_add(site.replenish)
                .ok_or("Food replenish total overflow")?;
        }
    }
    if food_site_positions.len() != stocks.len() {
        return Err("Food cell and Food site sets differ".into());
    }
    Ok((stocks, capacities, replenish))
}

/// Concentrate Food onto 8 deterministic 16-site patches while preserving
/// total initial stock, potential site capacity, and nominal replenish sum.
/// No World is constructed or mutated by this transform.
pub fn redistribute_food(source: &Scenario) -> Result<(Scenario, PatchReport), String> {
    let (food_stocks, old_capacity, old_replenish) = stock_and_sites(source)?;
    if food_stocks.len() < PATCH_SITES {
        return Err("not enough original Food sites for eight 16-site patches".into());
    }
    let source_food_total = food_stocks.values().try_fold(0usize, |sum, value| {
        sum.checked_add(*value).ok_or("initial Food total overflow")
    })?;
    let food_positions = food_stocks.keys().copied().collect::<BTreeSet<_>>();
    let protected_positions = source
        .resources
        .iter()
        .filter_map(|(pos, cell)| {
            matches!(cell, Cell::Water { .. } | Cell::Burrow | Cell::Wall).then_some(tuple(*pos))
        })
        .chain(source.founders.iter().map(|f| tuple(f.pos)))
        .collect::<BTreeSet<_>>();
    if food_positions
        .iter()
        .any(|position| protected_positions.contains(position))
    {
        return Err("Food cells overlap a founder or protected map cell".into());
    }
    let patch_anchors = anchors(source.bounds)?;
    let mut remaining = food_positions.clone();
    let mut patches: Vec<Vec<Position>> = Vec::with_capacity(patch_anchors.len());
    for anchor in &patch_anchors {
        let mut candidates = remaining.iter().copied().collect::<Vec<_>>();
        candidates.sort_by_key(|&(x, y)| {
            let pos = Position::new(x, y);
            (manhattan(pos, *anchor), y, x)
        });
        let chosen = candidates
            .into_iter()
            .take(SITES_PER_PATCH)
            .map(|(x, y)| Position::new(x, y))
            .collect::<Vec<_>>();
        if chosen.len() != SITES_PER_PATCH {
            return Err("Food patch selection did not fill a 16-site patch".into());
        }
        for position in &chosen {
            remaining.remove(&tuple(*position));
        }
        patches.push(chosen);
    }
    let selected: BTreeSet<_> = patches.iter().flatten().map(|p| tuple(*p)).collect();
    if selected.len() != PATCH_SITES {
        return Err("Food patches are not disjoint".into());
    }

    let mut target = Scenario {
        name: source.name.clone(),
        bounds: source.bounds,
        founders: source
            .founders
            .iter()
            .map(|f| curious::scenario::Founder {
                species: f.species.clone(),
                pos: f.pos,
                genome: f.genome.clone(),
                life: f.life.clone(),
            })
            .collect(),
        sites: source.sites.clone(),
        resources: source.resources.clone(),
        genetics: source.genetics.clone(),
        policy: source.policy.clone(),
    };

    // Preserve resource-row order and all non-Food cells exactly.
    for (_pos, cell) in &mut target.resources {
        if matches!(cell, Cell::Food { .. }) {
            *cell = Cell::Empty;
        }
    }

    let mut ordered_sites = Vec::with_capacity(PATCH_SITES);
    for patch in &patches {
        let mut ordered = patch.clone();
        ordered.sort_by_key(|p| (p.y, p.x));
        ordered_sites.extend(ordered);
    }
    let patch_position_set = selected;
    let mut retained_food_sites = Vec::with_capacity(PATCH_SITES);
    target.sites.retain_mut(|site| {
        if site.kind != ResourceKind::Food {
            return true;
        }
        if !patch_position_set.contains(&tuple(site.pos)) {
            return false;
        }
        retained_food_sites.push((site.pos, site.clone()));
        true
    });

    let mut by_position = retained_food_sites
        .into_iter()
        .map(|(pos, site)| (tuple(pos), site))
        .collect::<BTreeMap<_, _>>();
    for (index, position) in ordered_sites.iter().enumerate() {
        let mut site = by_position
            .remove(&tuple(*position))
            .ok_or_else(|| format!("selected Food site missing at {position:?}"))?;
        if site.interval != 1 || site.schedule.is_some() {
            return Err("fixture Food sites must use interval 1 and no schedule".into());
        }
        site.capacity = if index < 64 { 123 } else { 122 };
        site.replenish = if index < 32 { 13 } else { 12 };
        let cell = target
            .resources
            .iter_mut()
            .find(|(pos, _)| tuple(*pos) == tuple(*position))
            .ok_or_else(|| format!("selected Food cell missing at {position:?}"))?;
        cell.1 = Cell::Food {
            amount: site.capacity,
        };
        let target_site = target
            .sites
            .iter_mut()
            .find(|candidate| {
                candidate.kind == ResourceKind::Food && tuple(candidate.pos) == tuple(*position)
            })
            .ok_or_else(|| format!("selected Food site row missing at {position:?}"))?;
        *target_site = site;
    }
    if !by_position.is_empty() {
        return Err("unassigned selected Food sites remain".into());
    }
    let (new_stocks, new_capacity, new_replenish) = stock_and_sites(&target)?;
    let new_food_total = new_stocks.values().try_fold(0usize, |sum, value| {
        sum.checked_add(*value)
            .ok_or("redistributed Food total overflow")
    })?;
    if new_stocks.len() != PATCH_SITES
        || new_food_total != source_food_total
        || new_capacity != old_capacity
        || new_replenish != old_replenish
    {
        return Err("Food transform did not preserve expected totals".into());
    }
    let report = PatchReport {
        anchors: patch_anchors.iter().copied().map(tuple).collect(),
        patch_positions: patches
            .iter()
            .map(|patch| patch.iter().copied().map(tuple).collect())
            .collect(),
        initial_food: new_food_total,
        food_site_count: new_stocks.len(),
        potential_capacity: new_capacity,
        nominal_replenish_per_tick: new_replenish,
        intervals: target
            .sites
            .iter()
            .filter(|s| s.kind == ResourceKind::Food)
            .fold(BTreeMap::new(), |mut counts, site| {
                *counts.entry(site.interval).or_insert(0) += 1;
                counts
            }),
    };
    Ok((target, report))
}

/// Static Manhattan geometry to the nearest plant site; this does not model
/// the engine's bounded BFS perception or vision discovery.
pub fn founder_geometry(scenario: &Scenario) -> Result<Vec<FounderGeometry>, String> {
    let (food, _, _) = stock_and_sites(scenario)?;
    if food.is_empty() {
        return Err("no Food sites for founder geometry".into());
    }
    let food_positions = food
        .keys()
        .map(|&(x, y)| Position::new(x, y))
        .collect::<Vec<_>>();
    let mut grouped = BTreeMap::<String, Vec<usize>>::new();
    for founder in &scenario.founders {
        if founder.species != curious::entity::species::Species::Herbivore {
            continue;
        }
        let genotype = format!(
            "herbivore:m{}v{}s{}",
            founder.genome.mobility, founder.genome.vision, founder.genome.stamina
        );
        let distance = food_positions
            .iter()
            .map(|p| manhattan(founder.pos, *p))
            .min()
            .ok_or("no Food site for founder geometry")?;
        grouped.entry(genotype).or_default().push(distance);
    }
    Ok(grouped
        .into_iter()
        .map(|(genotype, distances)| {
            let mut counts = BTreeMap::new();
            for distance in &distances {
                *counts.entry(*distance).or_insert(0) += 1;
            }
            FounderGeometry {
                genotype,
                founders: distances.len(),
                within_base_range_6: distances.iter().filter(|d| **d <= 6).count(),
                at_distance_7: distances.iter().filter(|d| **d == 7).count(),
                nearest_food_distance_counts: counts,
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use curious::{
        entity::{genetics::Genome, species::Species},
        environment::ResourceSite,
        scenario::Founder,
    };

    fn fixture() -> Scenario {
        let mut sites = Vec::new();
        let mut resources = Vec::new();
        let mut site_index = 0usize;
        for y in (0..32).step_by(2) {
            for x in (0..32).step_by(2) {
                let pos = Position::new(x, y);
                let capacity = if site_index < 64 { 62 } else { 61 };
                let replenish = if site_index < 32 { 7 } else { 6 };
                sites.push(ResourceSite {
                    pos,
                    kind: ResourceKind::Food,
                    capacity,
                    replenish,
                    interval: 1,
                    schedule: None,
                });
                resources.push((pos, Cell::Food { amount: capacity }));
                site_index += 1;
            }
        }
        let water_pos = Position::new(1, 1);
        sites.push(ResourceSite {
            pos: water_pos,
            kind: ResourceKind::Water,
            capacity: 30,
            replenish: 3,
            interval: 2,
            schedule: None,
        });
        resources.push((water_pos, Cell::Water { amount: 10 }));
        resources.push((Position::new(3, 3), Cell::Burrow));
        resources.push((Position::new(5, 5), Cell::Wall));
        Scenario {
            name: "pure food patch fixture".into(),
            bounds: [0, 31, 0, 31],
            founders: vec![
                Founder {
                    species: Species::Herbivore,
                    pos: Position::new(1, 3),
                    genome: Genome {
                        mobility: 0,
                        vision: 0,
                        stamina: 0,
                    },
                    life: None,
                },
                Founder {
                    species: Species::Herbivore,
                    pos: Position::new(31, 31),
                    genome: Genome {
                        mobility: 0,
                        vision: 1,
                        stamina: 0,
                    },
                    life: None,
                },
            ],
            sites,
            resources,
            genetics: Default::default(),
            policy: Default::default(),
        }
    }

    fn food_total(scenario: &Scenario) -> usize {
        scenario
            .resources
            .iter()
            .map(|(_, cell)| match cell {
                Cell::Food { amount } => *amount,
                _ => 0,
            })
            .sum()
    }

    fn site_totals(scenario: &Scenario) -> (usize, usize, usize) {
        scenario
            .sites
            .iter()
            .filter(|site| site.kind == ResourceKind::Food)
            .fold((0, 0, 0), |(n, cap, refill), site| {
                (n + 1, cap + site.capacity, refill + site.replenish)
            })
    }

    #[test]
    fn transform_preserves_food_and_potential_site_totals_and_nonfood() {
        let source = fixture();
        let (patched, report) = redistribute_food(&source).unwrap();
        assert_eq!(food_total(&source), food_total(&patched));
        assert_eq!(site_totals(&source), (256, 15_680, 1_568));
        assert_eq!(site_totals(&patched), (128, 15_680, 1_568));
        assert_eq!(
            report.anchors,
            vec![
                (4, 8),
                (12, 8),
                (20, 8),
                (28, 8),
                (4, 24),
                (12, 24),
                (20, 24),
                (28, 24)
            ]
        );
        assert!(report.patch_positions.iter().all(|patch| patch.len() == 16));
        let old_nonfood = source
            .resources
            .iter()
            .filter(|(_, c)| !matches!(c, Cell::Food { .. }))
            .map(|(p, c)| (tuple(*p), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let new_cells = patched
            .resources
            .iter()
            .map(|(p, c)| (tuple(*p), c))
            .collect::<BTreeMap<_, _>>();
        assert!(
            old_nonfood
                .iter()
                .all(|(position, cell)| new_cells.get(position) == Some(&cell))
        );
        assert_eq!(source.founders.len(), patched.founders.len());
        assert_eq!(source.bounds, patched.bounds);
        assert_eq!(source.name, patched.name);
        assert_eq!(
            serde_json::to_value(&source.founders).unwrap(),
            serde_json::to_value(&patched.founders).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&source.policy).unwrap(),
            serde_json::to_value(&patched.policy).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&source.genetics).unwrap(),
            serde_json::to_value(&patched.genetics).unwrap()
        );
        let old_nonfood_sites = source
            .sites
            .iter()
            .filter(|site| site.kind != ResourceKind::Food)
            .collect::<Vec<_>>();
        let new_nonfood_sites = patched
            .sites
            .iter()
            .filter(|site| site.kind != ResourceKind::Food)
            .collect::<Vec<_>>();
        assert_eq!(
            serde_json::to_value(old_nonfood_sites).unwrap(),
            serde_json::to_value(new_nonfood_sites).unwrap()
        );
        assert_eq!(
            patched
                .resources
                .iter()
                .filter(|(_, cell)| matches!(cell, Cell::Empty))
                .count(),
            128
        );
        assert_eq!(report.intervals.get(&1), Some(&128));
    }

    #[test]
    fn selected_patches_are_unique_and_do_not_overlap_protected_map_cells() {
        let source = fixture();
        let (patched, report) = redistribute_food(&source).unwrap();
        let selected = report
            .patch_positions
            .iter()
            .flatten()
            .copied()
            .collect::<BTreeSet<_>>();
        assert_eq!(selected.len(), PATCH_SITES);
        for pos in selected {
            assert!(matches!(
                patched
                    .resources
                    .iter()
                    .find(|(candidate, _)| tuple(*candidate) == pos)
                    .map(|(_, cell)| cell),
                Some(Cell::Food { .. })
            ));
        }
        assert_eq!(report.patch_positions.len(), 8);
    }

    #[test]
    fn founder_geometry_groups_balanced_alleles_and_counts_ranges() {
        let source = fixture();
        let (patched, _) = redistribute_food(&source).unwrap();
        let accepted = founder_geometry(&source).unwrap();
        let redistributed = founder_geometry(&patched).unwrap();
        assert_eq!(accepted.len(), 2);
        assert_eq!(redistributed.len(), 2);
        assert!(accepted.iter().all(|entry| entry.founders == 1));
        assert!(redistributed.iter().all(|entry| entry.founders == 1));
    }

    #[test]
    fn rejects_mismatched_food_cells_and_sites() {
        let mut source = fixture();
        let index = source
            .sites
            .iter()
            .position(|site| site.kind == ResourceKind::Food)
            .unwrap();
        source.sites.remove(index);
        assert!(redistribute_food(&source).is_err());
    }

    #[test]
    fn rejects_food_overlap_with_founder_or_protected_cell() {
        let mut source = fixture();
        source.founders[0].pos = Position::new(2, 0);
        assert!(redistribute_food(&source).is_err());
    }
}
