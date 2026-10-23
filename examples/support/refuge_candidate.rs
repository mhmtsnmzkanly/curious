//! Isolated prototype for guarded reuse of refuge outcomes.
//!
//! This module is intentionally not wired into `World`, `Memory`, checkpoint
//! state, or a lab runner. Callers must provide an explicitly reviewed
//! compatibility gate and evidence support minimum.

use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreatKind {
    Carnivore,
    Omnivore,
}

/// Coarse observable features produced from current perception by a caller.
/// The reusable key deliberately has no individual threat ID or exact position.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Situation {
    pub threat: ThreatKind,
    pub bearing_sector: u8,
    pub distance_band: u8,
    pub energy_band: u8,
    pub stamina_band: u8,
    pub habitat_class: u8,
    // This prototype omits subject species and speed. Production integration
    // must add them or preregister and validate why cross-subject transfer is
    // safe; this helper does not claim all species/speeds are interchangeable.
}

/// Current perceived route properties. `terrain` contains only observed cells
/// along this route; endpoint coordinates and the alternative set are omitted.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Route {
    pub refuge_kind: u8,
    pub length: u16,
    pub turns: u8,
    pub approach_sector: u8,
    pub terrain: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CandidateKey {
    pub situation: Situation,
    pub route: Route,
}

/// A review-selected gate. The prototype has no implicit compatibility policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompatibilityGate {
    /// Maximum number of allowed differences among bearing, distance, energy,
    /// and stamina bands. Species and habitat always require an exact match.
    pub max_situation_band_differences: u8,
    /// Maximum absolute route length difference.
    pub max_route_length_difference: u16,
    /// Maximum absolute turn-count difference.
    pub max_turn_difference: u8,
}

pub fn compatible(query: &CandidateKey, observed: &CandidateKey, gate: CompatibilityGate) -> bool {
    let q = &query.situation;
    let o = &observed.situation;
    if q.threat != o.threat || q.habitat_class != o.habitat_class {
        return false;
    }
    let band_differences = [
        q.bearing_sector.abs_diff(o.bearing_sector),
        q.distance_band.abs_diff(o.distance_band),
        q.energy_band.abs_diff(o.energy_band),
        q.stamina_band.abs_diff(o.stamina_band),
    ]
    .into_iter()
    .filter(|difference| *difference != 0)
    .count() as u8;
    if band_differences > gate.max_situation_band_differences {
        return false;
    }

    let qr = &query.route;
    let or = &observed.route;
    qr.refuge_kind == or.refuge_kind
        && qr.approach_sector == or.approach_sector
        && qr.terrain == or.terrain
        && qr.length.abs_diff(or.length) <= gate.max_route_length_difference
        && qr.turns.abs_diff(or.turns) <= gate.max_turn_difference
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CensorReason {
    OtherAttacker,
    UnattributedHealthLoss,
    Death,
    ThreatLost,
    Timeout,
    InvalidRoute,
    Interrupted,
    AmbiguousArrival,
    LabBoundary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpisodeEvidence {
    /// Resolver-verified arrival with complete source attribution.
    Completed {
        threat_damage: u64,
        movement_energy: u64,
    },
    /// Audited closure that is excluded from reusable estimates.
    Censored(CensorReason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub key: CandidateKey,
    pub last_completed_tick: u64,
    pub completed: u32,
    pub safe_arrivals: u32,
    pub threat_damage: u64,
    pub movement_energy: u64,
}

impl Outcome {
    fn record(&mut self, evidence: EpisodeEvidence, tick: u64) -> bool {
        let EpisodeEvidence::Completed {
            threat_damage,
            movement_energy,
        } = evidence
        else {
            return false;
        };
        // Keep every field on a common denominator. If any aggregate cannot
        // accept this episode, freeze the entire row unchanged.
        let Some(completed) = self.completed.checked_add(1) else {
            return false;
        };
        let Some(safe_arrivals) = self
            .safe_arrivals
            .checked_add(u32::from(threat_damage == 0))
        else {
            return false;
        };
        let Some(total_threat_damage) = self.threat_damage.checked_add(threat_damage) else {
            return false;
        };
        let Some(total_movement_energy) = self.movement_energy.checked_add(movement_energy) else {
            return false;
        };
        self.last_completed_tick = tick;
        self.completed = completed;
        self.safe_arrivals = safe_arrivals;
        self.threat_damage = total_threat_damage;
        self.movement_energy = total_movement_energy;
        true
    }
}

/// Store only completed, attributed arrivals. Censored episodes remain in the
/// caller's audit ledger and never become zero-risk observations.
pub fn record_outcome(
    records: &mut Vec<Outcome>,
    key: CandidateKey,
    evidence: EpisodeEvidence,
    tick: u64,
    capacity: usize,
) -> bool {
    if !matches!(evidence, EpisodeEvidence::Completed { .. }) {
        return false;
    }
    if capacity == 0 {
        records.clear();
        return false;
    }
    let index = records.iter().position(|row| row.key == key);
    if let Some(index) = index {
        let updated = records[index].record(evidence, tick);
        // Reduce an oversized table by last-completion time and stable key;
        // the updated row may still be evicted if its recorded time is oldest.
        while records.len() > capacity {
            let oldest = oldest_record_index(records);
            records.remove(oldest);
        }
        return updated;
    }

    // For a new key, make room before insertion. Oldest completion is evicted
    // first, with stable key order as a deterministic tie-break.
    while records.len() >= capacity {
        let oldest = oldest_record_index(records);
        records.remove(oldest);
    }
    records.push(Outcome {
        key,
        last_completed_tick: tick,
        completed: 1,
        safe_arrivals: u32::from(matches!(
            evidence,
            EpisodeEvidence::Completed {
                threat_damage: 0,
                ..
            }
        )),
        threat_damage: match evidence {
            EpisodeEvidence::Completed { threat_damage, .. } => threat_damage,
            EpisodeEvidence::Censored(_) => unreachable!("censored evidence returned above"),
        },
        movement_energy: match evidence {
            EpisodeEvidence::Completed {
                movement_energy, ..
            } => movement_energy,
            EpisodeEvidence::Censored(_) => unreachable!("censored evidence returned above"),
        },
    });
    true
}

fn oldest_record_index(records: &[Outcome]) -> usize {
    records
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            (a.last_completed_tick, &a.key).cmp(&(b.last_completed_tick, &b.key))
        })
        .map(|(index, _)| index)
        .expect("oldest record requested only for a nonempty table")
}

/// Rank current candidates only when every candidate has compatible evidence
/// meeting the caller's support gate. `None` means use the existing fallback.
/// Returned indices are into `candidates`, best first, with input order as the
/// final deterministic tie-break.
pub fn rank_candidates(
    candidates: &[CandidateKey],
    records: &[Outcome],
    gate: CompatibilityGate,
    minimum_completed: u32,
) -> Option<Vec<usize>> {
    if candidates.is_empty() || minimum_completed == 0 {
        return None;
    }
    let mut ranked = Vec::with_capacity(candidates.len());
    let mut used_records = std::collections::BTreeSet::new();
    for (candidate_index, candidate) in candidates.iter().enumerate() {
        let mut matching = records.iter().enumerate().filter(|(_, row)| {
            row.completed >= minimum_completed && compatible(candidate, &row.key, gate)
        });
        let (record_index, first) = matching.next()?;
        // Multiple compatible rows would make evidence aggregation ambiguous;
        // fail closed until a reviewed aggregation rule exists.
        if matching.next().is_some() {
            return None;
        }
        // One stored route may not stand in for two currently distinct options.
        if !used_records.insert(record_index) {
            return None;
        }
        ranked.push((candidate_index, first));
    }
    ranked.sort_by(|(ai, a), (bi, b)| compare_outcomes(a, b).then_with(|| ai.cmp(bi)));
    Some(ranked.into_iter().map(|(index, _)| index).collect())
}

fn compare_outcomes(a: &Outcome, b: &Outcome) -> Ordering {
    // Higher safe-arrival rate, then lower mean threat damage, then lower mean
    // movement energy. Cross multiplication keeps ordering deterministic.
    let safety = (b.safe_arrivals as u128 * a.completed as u128)
        .cmp(&(a.safe_arrivals as u128 * b.completed as u128));
    if safety != Ordering::Equal {
        return safety;
    }
    let damage = (a.threat_damage as u128 * b.completed as u128)
        .cmp(&(b.threat_damage as u128 * a.completed as u128));
    if damage != Ordering::Equal {
        return damage;
    }
    (a.movement_energy as u128 * b.completed as u128)
        .cmp(&(b.movement_energy as u128 * a.completed as u128))
}
