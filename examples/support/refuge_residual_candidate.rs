//! Pure, unqualified per-individual refuge residual-model prototype.
//!
//! This helper is deliberately not connected to `World`, the learner, refuge
//! intent selection, checkpoint persistence, or RNG. Callers supply an
//! observable context, a per-route physical baseline, and a resolver-derived
//! episode outcome. Returned rankings are diagnostics only.

use std::collections::VecDeque;

pub const MAX_ROUTE_ROWS: usize = 8;
pub const MAX_PROVENANCE_ROWS: usize = 32;
pub const MAX_UPDATES: u32 = 256;
pub const FEATURE_COUNT: usize = 9;
const Q16: i64 = 1 << 16;
const MAX_RATIO: u64 = 8;
const MAX_PHYSICAL_VALUE: u32 = 1_000_000;
const MAX_RESIDUAL_UNITS: i64 = 256;
const MAX_WEIGHT_Q16: i64 = MAX_RESIDUAL_UNITS * Q16;
const LEARNING_RATE_DENOMINATOR: i64 = 16;

/// Route identity is deliberately small and exact. Each model instance belongs
/// to one owner, so no row can be reused by another individual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RouteKey {
    pub endpoint_x: i32,
    pub endpoint_y: i32,
    pub refuge_kind: RefugeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RefugeKind {
    Burrow,
}

/// Raw current-observation inputs. Ratios are converted to Q16 without
/// bucketization. Invalid or excessively large ratios fail closed to unknown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObservableContext {
    pub subject_health: u32,
    pub subject_max_health: u32,
    pub subject_energy: u32,
    pub subject_max_energy: u32,
    pub subject_stamina: u32,
    pub subject_max_stamina: u32,
    pub threat_distance_steps: u32,
    /// Current subject's known vision radius, not a target property.
    pub subject_vision_range: u32,
    pub threat_power: u32,
    /// Health may be absent in perception; absence makes the feature vector
    /// incomplete rather than inventing hidden target capacity.
    pub threat_health: Option<u32>,
    pub route_steps: u32,
    pub route_turns: u32,
    pub estimated_movement_energy: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureVector([i32; FEATURE_COUNT]);

impl ObservableContext {
    pub fn features(self) -> Option<FeatureVector> {
        if self.route_steps == 0 || self.route_turns > self.route_steps {
            return None;
        }
        let values = [
            ratio(self.subject_health, self.subject_max_health)?,
            ratio(self.subject_energy, self.subject_max_energy)?,
            ratio(self.subject_stamina, self.subject_max_stamina)?,
            ratio(self.threat_distance_steps, self.subject_vision_range)?,
            // Subject maximum health is an explicitly observable arithmetic
            // scale. It does not imply physical equivalence across threats.
            ratio(self.threat_power, self.subject_max_health)?,
            ratio(self.threat_health?, self.subject_max_health)?,
            ratio(self.route_steps, self.subject_vision_range)?,
            ratio(self.route_turns, self.route_steps)?,
            ratio(self.estimated_movement_energy, self.subject_max_energy)?,
        ];
        Some(FeatureVector(values))
    }
}

fn ratio(numerator: u32, denominator: u32) -> Option<i32> {
    if denominator == 0 {
        return None;
    }
    let scaled = u64::from(numerator).checked_mul(Q16 as u64)? / u64::from(denominator);
    if scaled > MAX_RATIO * Q16 as u64 {
        return None;
    }
    i32::try_from(scaled).ok()
}

/// Separate physical components; these are never collapsed into a reward.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhysicalVector {
    pub threat_damage: u32,
    pub paid_movement_energy: u32,
    pub elapsed_ticks: u32,
}

impl PhysicalVector {
    fn within_bounds(self) -> bool {
        self.threat_damage <= MAX_PHYSICAL_VALUE
            && self.paid_movement_energy <= MAX_PHYSICAL_VALUE
            && self.elapsed_ticks <= MAX_PHYSICAL_VALUE
    }

    fn components(self) -> [u32; 3] {
        [
            self.threat_damage,
            self.paid_movement_energy,
            self.elapsed_ticks,
        ]
    }

    fn from_components(values: [u32; 3]) -> Self {
        Self {
            threat_damage: values[0],
            paid_movement_energy: values[1],
            elapsed_ticks: values[2],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaselineCalibration {
    /// A baseline is supplied for mechanics tests, but has no empirical
    /// calibration. This must never be described as an expected physical law.
    Uncalibrated,
    /// An external calibration identifier supplied by a future integration.
    ExternallyCalibrated { identifier: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteCandidate {
    pub key: RouteKey,
    pub features: Option<FeatureVector>,
    /// Missing baseline components make the route unrankable by the model.
    pub baseline: Option<PhysicalVector>,
    pub baseline_calibration: BaselineCalibration,
    /// Stable current-rule route order for deterministic ties and default-off.
    pub baseline_order: usize,
}

impl RouteCandidate {
    pub fn from_observation(
        key: RouteKey,
        context: ObservableContext,
        baseline: Option<PhysicalVector>,
        baseline_calibration: BaselineCalibration,
        baseline_order: usize,
    ) -> Self {
        Self {
            key,
            features: context.features(),
            baseline,
            baseline_calibration,
            baseline_order,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Closure {
    Arrived,
    ThreatLost,
    Died,
    TimedOut,
    Interrupted,
    InvalidRoute,
    Ambiguous,
}

/// Resolver-measured closure data. A nonzero other/unattributed damage field,
/// missing threat ID, or non-arrival closure censors the whole sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompletedEpisode {
    pub owner_id: u64,
    pub route: RouteKey,
    pub features: Option<FeatureVector>,
    pub baseline: Option<PhysicalVector>,
    pub actual: PhysicalVector,
    pub closure: Closure,
    pub threat_id: Option<u64>,
    pub other_damage: u32,
    pub unexplained_health_loss: u32,
    pub ended_tick: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateResult {
    Applied { clipped_components: u8 },
    Censored,
    Disabled,
    Frozen,
    WrongOwner,
    MissingFeaturesOrBaseline,
    InvalidPhysicalValue,
    RouteLimit,
    UpdateLimit,
    ArithmeticLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Provenance {
    pub route: RouteKey,
    pub threat_id: u64,
    pub ended_tick: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LinearOutput {
    /// Bias then one coefficient for each Q16 feature. Values are Q16 physical
    /// units per feature unit.
    weights: [i64; FEATURE_COUNT + 1],
}

impl Default for LinearOutput {
    fn default() -> Self {
        Self {
            weights: [0; FEATURE_COUNT + 1],
        }
    }
}

impl LinearOutput {
    fn predict_q16(self, features: FeatureVector) -> Result<i64, ()> {
        let mut total = self.weights[0];
        for (weight, feature) in self.weights[1..].iter().zip(features.0) {
            let product = weight.checked_mul(i64::from(feature)).ok_or(())?;
            total = total.checked_add(product / Q16).ok_or(())?;
        }
        Ok(total.clamp(-MAX_WEIGHT_Q16, MAX_WEIGHT_Q16))
    }

    fn updated(self, features: FeatureVector, target_q16: i64) -> Result<(Self, bool), ()> {
        let prediction = self.predict_q16(features)?;
        let raw_error = target_q16.checked_sub(prediction).ok_or(())?;
        let clipped = raw_error.clamp(-MAX_RESIDUAL_UNITS * Q16, MAX_RESIDUAL_UNITS * Q16);
        let was_clipped = raw_error != clipped;
        let mut next = self;
        let bias_delta = clipped / LEARNING_RATE_DENOMINATOR;
        next.weights[0] = bounded_weight(next.weights[0], bias_delta)?;
        for (index, feature) in features.0.iter().enumerate() {
            let numerator = clipped.checked_mul(i64::from(*feature)).ok_or(())?;
            let delta = numerator / (Q16 * LEARNING_RATE_DENOMINATOR);
            next.weights[index + 1] = bounded_weight(next.weights[index + 1], delta)?;
        }
        Ok((next, was_clipped))
    }
}

fn bounded_weight(current: i64, delta: i64) -> Result<i64, ()> {
    current
        .checked_add(delta)
        .filter(|value| value.abs() <= MAX_WEIGHT_Q16)
        .ok_or(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RouteModel {
    key: RouteKey,
    outputs: [LinearOutput; 3],
    updates: u32,
}

impl RouteModel {
    fn new(key: RouteKey) -> Self {
        Self {
            key,
            outputs: [LinearOutput::default(); 3],
            updates: 0,
        }
    }
}

/// One individual's bounded, default-off online residual predictor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResidualModel {
    owner_id: u64,
    enabled: bool,
    frozen: bool,
    total_updates: u32,
    routes: Vec<RouteModel>,
    provenance: VecDeque<Provenance>,
}

impl ResidualModel {
    pub fn new(owner_id: u64) -> Self {
        Self {
            owner_id,
            enabled: false,
            frozen: false,
            total_updates: 0,
            routes: Vec::new(),
            provenance: VecDeque::new(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_frozen(&mut self, frozen: bool) {
        self.frozen = frozen;
    }

    pub fn total_updates(&self) -> u32 {
        self.total_updates
    }

    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    pub fn provenance(&self) -> &VecDeque<Provenance> {
        &self.provenance
    }

    pub fn update(&mut self, episode: CompletedEpisode) -> UpdateResult {
        if !self.enabled {
            return UpdateResult::Disabled;
        }
        if self.frozen {
            return UpdateResult::Frozen;
        }
        if episode.owner_id != self.owner_id {
            return UpdateResult::WrongOwner;
        }
        if episode.closure != Closure::Arrived
            || episode.other_damage != 0
            || episode.unexplained_health_loss != 0
        {
            return UpdateResult::Censored;
        }
        let Some(threat_id) = episode.threat_id else {
            return UpdateResult::Censored;
        };
        let (Some(features), Some(baseline)) = (episode.features, episode.baseline) else {
            return UpdateResult::MissingFeaturesOrBaseline;
        };
        if !baseline.within_bounds() || !episode.actual.within_bounds() {
            return UpdateResult::InvalidPhysicalValue;
        }
        if self.total_updates >= MAX_UPDATES {
            return UpdateResult::UpdateLimit;
        }

        let existing = self
            .routes
            .iter()
            .position(|route| route.key == episode.route);
        if existing.is_none() && self.routes.len() >= MAX_ROUTE_ROWS {
            return UpdateResult::RouteLimit;
        }
        let mut staged = existing
            .map(|index| self.routes[index])
            .unwrap_or_else(|| RouteModel::new(episode.route));
        let baseline_values = baseline.components();
        let actual_values = episode.actual.components();
        let mut clipped_components = 0;
        for output_index in 0..3 {
            let residual = i64::from(actual_values[output_index])
                .checked_sub(i64::from(baseline_values[output_index]))
                .and_then(|value| value.checked_mul(Q16));
            let Some(residual) = residual else {
                return UpdateResult::ArithmeticLimit;
            };
            let Ok((next, clipped)) = staged.outputs[output_index].updated(features, residual)
            else {
                return UpdateResult::ArithmeticLimit;
            };
            staged.outputs[output_index] = next;
            clipped_components += u8::from(clipped);
        }
        let Some(next_route_updates) = staged.updates.checked_add(1) else {
            return UpdateResult::ArithmeticLimit;
        };
        staged.updates = next_route_updates;
        let Some(next_total_updates) = self.total_updates.checked_add(1) else {
            return UpdateResult::ArithmeticLimit;
        };

        if let Some(index) = existing {
            self.routes[index] = staged;
        } else {
            self.routes.push(staged);
        }
        self.total_updates = next_total_updates;
        if self.provenance.len() == MAX_PROVENANCE_ROWS {
            self.provenance.pop_front();
        }
        self.provenance.push_back(Provenance {
            route: episode.route,
            threat_id,
            ended_tick: episode.ended_tick,
        });
        UpdateResult::Applied { clipped_components }
    }

    pub fn query(&self, owner_id: u64, candidates: &[RouteCandidate]) -> QueryReport {
        let owner_matches = owner_id == self.owner_id;
        let active = self.enabled && owner_matches;
        let mut predictions = Vec::with_capacity(candidates.len());
        let mut cannot_rank = false;

        for candidate in candidates {
            if active && candidate.baseline.is_some_and(|base| !base.within_bounds()) {
                cannot_rank = true;
                predictions.push(RoutePrediction {
                    key: candidate.key,
                    baseline: candidate.baseline,
                    prediction: candidate.baseline,
                    baseline_calibration: candidate.baseline_calibration.clone(),
                    status: PredictionStatus::Unknown,
                    residual_q16: None,
                    updates: 0,
                    baseline_order: candidate.baseline_order,
                });
                continue;
            }
            let model = if active {
                self.routes.iter().find(|route| route.key == candidate.key)
            } else {
                None
            };
            let (prediction, status, residual_q16, updates) =
                match (candidate.baseline, candidate.features, model) {
                    (Some(baseline), Some(features), Some(route)) => {
                        match predict_vector(route, features, baseline) {
                            Some((prediction, residual_q16)) => (
                                Some(prediction),
                                PredictionStatus::Provisional {
                                    updates: route.updates,
                                },
                                Some(residual_q16),
                                route.updates,
                            ),
                            None => {
                                cannot_rank = true;
                                (Some(baseline), PredictionStatus::Unknown, None, 0)
                            }
                        }
                    }
                    (Some(baseline), Some(_), None) => {
                        (Some(baseline), PredictionStatus::Unknown, None, 0)
                    }
                    (Some(baseline), None, _) => {
                        (Some(baseline), PredictionStatus::Unknown, None, 0)
                    }
                    (None, _, _) => {
                        cannot_rank = true;
                        (None, PredictionStatus::Unknown, None, 0)
                    }
                };
            predictions.push(RoutePrediction {
                key: candidate.key,
                baseline: candidate.baseline,
                prediction,
                baseline_calibration: candidate.baseline_calibration.clone(),
                status,
                residual_q16,
                updates,
                baseline_order: candidate.baseline_order,
            });
        }

        if cannot_rank {
            for prediction in &mut predictions {
                prediction.prediction = prediction.baseline;
                prediction.status = PredictionStatus::Unknown;
                prediction.residual_q16 = None;
                prediction.updates = 0;
            }
        }
        let baseline_index = baseline_choice(candidates);
        let has_physical_adjustment = predictions.iter().any(|row| {
            matches!((row.baseline, row.prediction), (Some(base), Some(prediction)) if base != prediction)
        });
        let selected_index = if !active || cannot_rank || !has_physical_adjustment {
            baseline_index
        } else {
            diagnostic_choice(&predictions)
        };
        QueryReport {
            selected_index,
            baseline_index,
            predictions,
            model_enabled: self.enabled,
            owner_matches,
        }
    }
}

fn predict_vector(
    route: &RouteModel,
    features: FeatureVector,
    baseline: PhysicalVector,
) -> Option<(PhysicalVector, [i64; 3])> {
    if !baseline.within_bounds() {
        return None;
    }
    let baseline_values = baseline.components();
    let mut adjusted = [0; 3];
    let mut residuals = [0; 3];
    for index in 0..3 {
        let residual = route.outputs[index]
            .predict_q16(features)
            .ok()?
            .clamp(-MAX_RESIDUAL_UNITS * Q16, MAX_RESIDUAL_UNITS * Q16);
        residuals[index] = residual;
        let prediction = i64::from(baseline_values[index])
            .checked_mul(Q16)?
            .checked_add(residual)?;
        let physical = (prediction.max(0) + Q16 / 2) / Q16;
        adjusted[index] = u32::try_from(physical).ok()?.min(MAX_PHYSICAL_VALUE);
    }
    Some((PhysicalVector::from_components(adjusted), residuals))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PredictionStatus {
    Unknown,
    Provisional { updates: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutePrediction {
    pub key: RouteKey,
    pub baseline: Option<PhysicalVector>,
    pub prediction: Option<PhysicalVector>,
    pub baseline_calibration: BaselineCalibration,
    pub status: PredictionStatus,
    /// Physical residuals represented in Q16, kept separate per component.
    pub residual_q16: Option<[i64; 3]>,
    pub updates: u32,
    pub baseline_order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryReport {
    /// Diagnostic index only; no caller action is changed by this helper.
    pub selected_index: Option<usize>,
    pub baseline_index: Option<usize>,
    pub predictions: Vec<RoutePrediction>,
    pub model_enabled: bool,
    pub owner_matches: bool,
}

fn baseline_choice(candidates: &[RouteCandidate]) -> Option<usize> {
    candidates
        .iter()
        .enumerate()
        .min_by_key(|(_, candidate)| candidate.baseline_order)
        .map(|(index, _)| index)
}

fn diagnostic_choice(predictions: &[RoutePrediction]) -> Option<usize> {
    predictions
        .iter()
        .enumerate()
        .filter_map(|(index, prediction)| {
            prediction
                .prediction
                .map(|vector| (index, vector, prediction.baseline_order))
        })
        .min_by_key(|(_, vector, order)| {
            (
                vector.threat_damage,
                vector.paid_movement_energy,
                vector.elapsed_ticks,
                *order,
            )
        })
        .map(|(index, _, _)| index)
}
