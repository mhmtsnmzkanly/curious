//! Isolated score model for the parallel water policy design.
//!
//! This is not wired into `Learner` or `World`. Callers must only record a
//! residual after a completed, attributable episode and supply the source's
//! choice-time baseline plus resolver-derived outcome values.

use std::cmp::Reverse;

use curious::{
    entity::{foraging, lifestate::LifeState, perception::PerceivedWater},
    map::movement::Position,
};

// Match the current water episode's validated accounting limits. `MAX_WATER_TICKS`
// is public in the core learner; its per-account cap is currently private, so
// this isolated model names that same declared limit locally.
const MAX_EPISODE_TICKS: usize = curious::entity::learning::MAX_WATER_TICKS as usize;
const MAX_EPISODE_ACCOUNT: usize = 1_000_000;
const MAX_EPISODE_WATER: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceResidual {
    pub pos: Position,
    /// Mean difference from choice-time foraging score, in that score's units.
    pub residual: i64,
    pub samples: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CandidateScore {
    pub pos: Position,
    pub baseline: i64,
    pub residual: i64,
    pub total: i64,
    pub perception_index: usize,
    pub path_len: usize,
}

/// Calculate current score plus only this source's observed residual.
pub fn score_sources(
    origin: Position,
    waters: &[PerceivedWater],
    life: &LifeState,
    residuals: &[SourceResidual],
) -> Vec<CandidateScore> {
    waters
        .iter()
        .enumerate()
        .map(|(perception_index, water)| {
            let pos = water.steps.0.iter().fold(origin, |p, d| p + *d);
            let baseline = foraging::value(water.amount, water.steps.len(), life, true) as i64;
            let residual = residuals
                .iter()
                .find(|r| r.pos == pos && r.samples > 0)
                .map_or(0, |r| r.residual);
            CandidateScore {
                pos,
                baseline,
                residual,
                total: baseline.saturating_add(residual),
                perception_index,
                path_len: water.steps.len(),
            }
        })
        .collect()
}

/// Preserve the rule's route-length and perception-order tie breaks.
pub fn choose_index(scores: &[CandidateScore]) -> Option<usize> {
    scores
        .iter()
        .max_by_key(|score| {
            (
                score.total,
                Reverse(score.path_len),
                Reverse(score.perception_index),
            )
        })
        .map(|score| score.perception_index)
}

/// Default-off dispatch seam: when disabled, return the legacy learner's
/// already-computed choice without changing or drawing RNG.
pub fn select_index(
    candidate_enabled: bool,
    legacy_index: Option<usize>,
    scores: &[CandidateScore],
) -> Option<usize> {
    if candidate_enabled {
        choose_index(scores)
    } else {
        legacy_index
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreBoundsError {
    ZeroTicks,
    TooManyTicks,
    TooMuchWater,
    MovementAccountExceeded,
}

/// Outcome score in the same units as `entity::foraging::value`:
/// water is weighted by two, then resolver-paid movement energy and elapsed
/// ticks are charged, and the result is normalized per elapsed tick. `water`
/// and `movement_energy` must come from this subject's resolver events. Reject
/// values outside the current episode contract rather than silently changing
/// an observed cost or amount.
pub fn actual_score(
    water: usize,
    movement_energy: usize,
    elapsed_ticks: usize,
) -> Result<i64, ScoreBoundsError> {
    if elapsed_ticks == 0 {
        return Err(ScoreBoundsError::ZeroTicks);
    }
    if elapsed_ticks > MAX_EPISODE_TICKS {
        return Err(ScoreBoundsError::TooManyTicks);
    }
    if water > MAX_EPISODE_WATER {
        return Err(ScoreBoundsError::TooMuchWater);
    }
    if movement_energy > MAX_EPISODE_ACCOUNT {
        return Err(ScoreBoundsError::MovementAccountExceeded);
    }
    let ticks = elapsed_ticks as i128;
    let benefit = water as i128 * 2;
    let cost = movement_energy as i128 + ticks;
    Ok((((benefit - cost) * 100) / ticks) as i64)
}

/// Add one completed observation as a running-mean residual.
///
/// The caller intentionally cannot record an interrupted/censored attempt by
/// passing a flag; those paths should not call this function at all.
fn record_completed(
    residuals: &mut Vec<SourceResidual>,
    pos: Position,
    choice_time_baseline: i64,
    actual: i64,
) {
    // Bound an externally supplied or migrated oversized table before either
    // lookup/update or insertion. Keep the newest 32 entries in FIFO order.
    if residuals.len() > 32 {
        let excess = residuals.len() - 32;
        residuals.drain(0..excess);
    }
    let delta = actual.saturating_sub(choice_time_baseline);
    if let Some(entry) = residuals.iter_mut().find(|entry| entry.pos == pos) {
        let next = entry.samples.saturating_add(1);
        if next == entry.samples {
            return;
        }
        // Integer recursive mean approximation. The difference form avoids
        // accumulating an unbounded sum, but truncation can stall small
        // corrections after many samples; see the design note.
        let mean = entry.residual as i128;
        let updated = mean + (delta as i128 - mean) / next as i128;
        entry.residual = updated.clamp(i64::MIN as i128, i64::MAX as i128) as i64;
        entry.samples = next;
    } else {
        if residuals.len() == 32 {
            residuals.remove(0);
        }
        residuals.push(SourceResidual {
            pos,
            residual: delta,
            samples: 1,
        });
    }
}

/// Admit a drink outcome only when both the actor and its selected source
/// match. Resolver rejection/failure handling remains a separate caller path.
fn record_matching_drink(
    residuals: &mut Vec<SourceResidual>,
    subject_id: usize,
    event_actor_id: usize,
    selected_target: Position,
    event_target: Position,
    choice_time_baseline: i64,
    water: usize,
    movement_energy: usize,
    elapsed_ticks: usize,
) -> Result<bool, ScoreBoundsError> {
    if event_actor_id != subject_id || event_target != selected_target {
        return Ok(false);
    }
    let actual = actual_score(water, movement_energy, elapsed_ticks)?;
    record_completed(residuals, selected_target, choice_time_baseline, actual);
    Ok(true)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpisodeEvidence {
    Censored,
    CompletedDrink {
        subject_id: usize,
        event_actor_id: usize,
        selected_target: Position,
        event_target: Position,
        water: usize,
        movement_energy: usize,
        elapsed_ticks: usize,
    },
}

/// Admit only a completed, actor- and target-matched drink. Censored outcomes
/// are explicit and leave the learned table unchanged.
pub fn record_episode_evidence(
    residuals: &mut Vec<SourceResidual>,
    evidence: EpisodeEvidence,
    choice_time_baseline: i64,
) -> Result<bool, ScoreBoundsError> {
    match evidence {
        EpisodeEvidence::Censored => Ok(false),
        EpisodeEvidence::CompletedDrink {
            subject_id,
            event_actor_id,
            selected_target,
            event_target,
            water,
            movement_energy,
            elapsed_ticks,
        } => record_matching_drink(
            residuals,
            subject_id,
            event_actor_id,
            selected_target,
            event_target,
            choice_time_baseline,
            water,
            movement_energy,
            elapsed_ticks,
        ),
    }
}
