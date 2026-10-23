//! Offline prototype for narrowly guarded reuse of refuge outcomes.
//!
//! This module has no production imports. It permits transfer across a changed
//! threat identity or visible alternative set only at the selected-feature
//! level; coarse bands may still alias physically different observations.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Species {
    Herbivore,
    Carnivore,
    Omnivore,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Situation {
    /// Species of the individual making the choice.
    pub subject_species: Option<Species>,
    /// Species of the currently perceived threat.
    pub threat_species: Option<Species>,
    /// Observable, coarse context. These remain exact-match fields for now.
    pub bearing_sector: Option<u8>,
    pub distance_band: Option<u8>,
    pub energy_band: Option<u8>,
    pub stamina_band: Option<u8>,
    pub habitat_class: Option<u8>,
    /// Current speed, derived from the individual's own observable state.
    pub subject_speed: Option<u16>,
}

/// One visible step along a candidate route. Direction is relative to the
/// subject and terrain is the locally observed cell classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObservedStep {
    pub direction_sector: u8,
    pub terrain_class: u8,
}

/// Stable identity for one currently visible route, based on its observable
/// path. Distinct routes stay distinct even when their scalar lengths match.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RouteIdentity {
    pub endpoint_sector: u8,
    pub steps: Vec<ObservedStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Route {
    pub identity: RouteIdentity,
    pub refuge_kind: u8,
    pub turns: u8,
    /// Current estimated movement energy for this visible path. Actual energy
    /// paid remains a separate resolver outcome.
    pub estimated_movement_energy: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CandidateKey {
    pub situation: Situation,
    pub route: Route,
}

/// The initial policy is exact over this selected feature representation.
/// Coarse bands may alias different raw observations; that is an unvalidated
/// model risk. There are no caller-set radii or implicit thresholds.
pub fn compatible(query: &CandidateKey, observed: &CandidateKey) -> bool {
    let q = &query.situation;
    let o = &observed.situation;
    let fields_known = q.subject_species.is_some()
        && q.threat_species.is_some()
        && q.bearing_sector.is_some()
        && q.distance_band.is_some()
        && q.energy_band.is_some()
        && q.stamina_band.is_some()
        && q.habitat_class.is_some()
        && q.subject_speed.is_some()
        && query.route.estimated_movement_energy.is_some()
        && o.subject_species.is_some()
        && o.threat_species.is_some()
        && o.bearing_sector.is_some()
        && o.distance_band.is_some()
        && o.energy_band.is_some()
        && o.stamina_band.is_some()
        && o.habitat_class.is_some()
        && o.subject_speed.is_some()
        && observed.route.estimated_movement_energy.is_some();
    fields_known
        && q.subject_species == o.subject_species
        && q.threat_species == o.threat_species
        && q.bearing_sector == o.bearing_sector
        && q.distance_band == o.distance_band
        && q.energy_band == o.energy_band
        && q.stamina_band == o.stamina_band
        && q.habitat_class == o.habitat_class
        && q.subject_speed == o.subject_speed
        && query.route == observed.route
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

/// Store only completed attributed arrivals. Censored evidence stays in the
/// caller's audit ledger and never becomes a zero-risk observation.
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
    if let Some(index) = records.iter().position(|row| row.key == key) {
        let updated = records[index].record(evidence, tick);
        while records.len() > capacity {
            records.remove(oldest_record_index(records));
        }
        return updated;
    }
    while records.len() >= capacity {
        records.remove(oldest_record_index(records));
    }
    let EpisodeEvidence::Completed {
        threat_damage,
        movement_energy,
    } = evidence
    else {
        unreachable!("censored evidence returned above")
    };
    records.push(Outcome {
        key,
        last_completed_tick: tick,
        completed: 1,
        safe_arrivals: u32::from(threat_damage == 0),
        threat_damage,
        movement_energy,
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

/// Rank only when every current route has unique supported evidence. `None`
/// means preserve the existing fallback ordering for the whole choice.
pub fn rank_candidates(
    candidates: &[CandidateKey],
    records: &[Outcome],
    minimum_completed: u32,
) -> Option<Vec<usize>> {
    if candidates.is_empty() || minimum_completed == 0 {
        return None;
    }
    let mut ranked = Vec::with_capacity(candidates.len());
    let mut used_records = std::collections::BTreeSet::new();
    for (candidate_index, candidate) in candidates.iter().enumerate() {
        let mut matching = records.iter().enumerate().filter(|(_, row)| {
            row.completed >= minimum_completed && compatible(candidate, &row.key)
        });
        let (record_index, first) = matching.next()?;
        if matching.next().is_some() || !used_records.insert(record_index) {
            return None;
        }
        ranked.push((candidate_index, first));
    }
    ranked.sort_by(|(ai, a), (bi, b)| compare_outcomes(a, b).then_with(|| ai.cmp(bi)));
    Some(ranked.into_iter().map(|(index, _)| index).collect())
}

fn compare_outcomes(a: &Outcome, b: &Outcome) -> std::cmp::Ordering {
    let safe_rate = (b.safe_arrivals as u128 * a.completed as u128)
        .cmp(&(a.safe_arrivals as u128 * b.completed as u128));
    if safe_rate != std::cmp::Ordering::Equal {
        return safe_rate;
    }
    let damage = (a.threat_damage as u128 * b.completed as u128)
        .cmp(&(b.threat_damage as u128 * a.completed as u128));
    if damage != std::cmp::Ordering::Equal {
        return damage;
    }
    (a.movement_energy as u128 * b.completed as u128)
        .cmp(&(b.movement_energy as u128 * a.completed as u128))
}
