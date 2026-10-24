//! Fixed 270-physical-tick continuous-life refuge opportunity fixture.
//!
//! Do not run before root reviews docs/REFUGE_ESCAPE_PROTOCOL.md and this
//! source. The example accepts no seed or horizon arguments by design.
#[path = "support/refuge_escape_state.rs"]
mod state_audit;
#[path = "support/refuge_escape_capture.rs"]
mod support;

use curious::{
    entity::{memory::GeneralizedRefugeOutcome, phase::EntityPhase, species::Species},
    logger::Logger,
    scenario::Scenario,
    simulation::{SimulationEvent, checkpoint::State},
    world::World,
};
use serde::Serialize;
use serde_json::Value;
use state_audit::normalized_pre_world_state_bytes;
use std::io::{self, Write};
use support::{
    ChoiceWitness, FOCAL_ID, Provenance, UpdateSource, capture_choice, retain_fresh_rows,
};

const ENGINEERING_SEED: u64 = 123;
const PREFIX_TICKS: usize = 30;
const SUFFIX_TICKS: usize = 60;
const MAX_PHYSICAL_TICKS: usize = PREFIX_TICKS + SUFFIX_TICKS * 4;
const PROTOCOL: &str = "refuge_escape_continuous_life_v1_engineering";

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum Treatment {
    TrainedOnline,
    TrainedFrozen,
    NaiveOnline,
    RuleBased,
}

impl Treatment {
    const ALL: [Self; 4] = [
        Self::TrainedOnline,
        Self::TrainedFrozen,
        Self::NaiveOnline,
        Self::RuleBased,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::TrainedOnline => "trained_online",
            Self::TrainedFrozen => "trained_frozen",
            Self::NaiveOnline => "naive_online",
            Self::RuleBased => "rule_based",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct FocalState {
    id: usize,
    position: curious::map::movement::Position,
    phase: EntityPhase,
    life: curious::entity::lifestate::LifeState,
    refuge_generalization_enabled: bool,
    refuge_generalization_update_enabled: bool,
    active_episode: Option<curious::entity::memory::RefugeEpisode>,
    pending_closures: Vec<curious::entity::memory::PendingRefugeClosure>,
    generalized_outcomes: Vec<GeneralizedRefugeOutcome>,
}

#[derive(Clone, Debug, Serialize)]
struct TickRecord {
    scope: &'static str,
    arm: &'static str,
    suffix_tick: Option<usize>,
    world_tick: usize,
    physical_tick_ordinal: usize,
    rng_state_before_world_tick: u64,
    rng_state_after_world_tick: u64,
    world_state_fnv1a64_before: String,
    world_state_fnv1a64_after: String,
    full_pre_world_state_for_choice: Option<Value>,
    focal_before: Option<FocalState>,
    production_choice: Option<ChoiceWitness>,
    focal_events: Vec<Value>,
    focal_after: Option<FocalState>,
    table_changed_this_tick: bool,
    resolver_updates_this_tick: usize,
    provenance_replay_matches_table: bool,
    trained_table_differs_from_fork: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "record_type", rename_all = "snake_case")]
enum RawRecord {
    Header {
        protocol: &'static str,
        scenario: &'static str,
        seed: u64,
        focal_id: usize,
        prefix_ticks: usize,
        suffix_ticks_per_arm: usize,
        max_physical_ticks: usize,
        state_hash_algorithm: &'static str,
        evidence_role: &'static str,
    },
    InitialState {
        state: Value,
        rng_state: u64,
    },
    PrefixTick(TickRecord),
    Fork {
        state: Value,
        rng_state: u64,
        prefix_physical_ticks: usize,
        provenance: Vec<UpdateSource>,
    },
    TreatmentStart {
        arm: &'static str,
        focal_present: bool,
        state: Value,
        expected_only_treatment_difference: &'static str,
        inherited_active_episode: Option<curious::entity::memory::RefugeEpisode>,
    },
    SuffixTick(TickRecord),
    BranchEnd {
        arm: &'static str,
        physical_ticks: usize,
        focal_terminal_state: Option<FocalState>,
        table_at_fork: Vec<GeneralizedRefugeOutcome>,
        table_at_end: Vec<GeneralizedRefugeOutcome>,
        provenance_at_end: Vec<UpdateSource>,
        table_mutated: bool,
    },
    Summary {
        physical_ticks: usize,
        distinct_world_ticks: usize,
        focal_update_events: usize,
        post_prefix_supported_fresh_greedy_choices: usize,
        different_threat_id_supported_choices: usize,
        aligned_online_frozen_greedy_contrasts: usize,
        opportunity_gate: &'static str,
        interpretation: &'static str,
    },
}

struct CappedWriter<W> {
    inner: W,
    written: usize,
    limit: usize,
}

impl<W: Write> Write for CappedWriter<W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.written.saturating_add(buffer.len()) > self.limit {
            return Err(io::Error::other("raw output exceeded 128 MiB protocol cap"));
        }
        let written = self.inner.write(buffer)?;
        self.written += written;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

fn emit<W: Write, T: Serialize>(writer: &mut W, row: &T) -> io::Result<()> {
    let mut encoded = serde_json::to_vec(row)?;
    encoded.push(b'\n');
    writer.write_all(&encoded)?;
    writer.flush()
}

#[derive(Clone)]
struct ObservedChoice {
    suffix_tick: usize,
    witness: ChoiceWitness,
    normalized_pre_world_state: Vec<u8>,
    focal_events: Vec<Value>,
}

fn focal_state(world: &World) -> Option<FocalState> {
    world
        .entities
        .iter()
        .find(|slot| slot.id == FOCAL_ID)
        .map(|slot| FocalState {
            id: slot.id,
            position: slot.pos,
            phase: slot.phase,
            life: slot.base.life().clone(),
            refuge_generalization_enabled: slot.memory.refuge_generalization_enabled,
            refuge_generalization_update_enabled: slot.memory.refuge_generalization_update_enabled,
            active_episode: slot.memory.refuge_episode.clone(),
            pending_closures: slot.memory.pending_refuge_closures.clone(),
            generalized_outcomes: slot.memory.refuge_generalized_outcomes.clone(),
        })
}

fn fnv1a64(state: &State) -> String {
    let bytes = serde_json::to_vec(state).expect("serialize checkpoint for state witness");
    let hash = bytes.into_iter().fold(0xcbf29ce484222325u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    });
    format!("{hash:016x}")
}

fn focal_events(events: &[SimulationEvent]) -> Vec<Value> {
    events
        .iter()
        .filter(|event| match event {
            SimulationEvent::RefugeEpisodeEnded { id, .. }
            | SimulationEvent::EntityMoved { id, .. }
            | SimulationEvent::EntityAte { id, .. }
            | SimulationEvent::EntityDrank { id, .. }
            | SimulationEvent::EntityMetabolized { id, .. }
            | SimulationEvent::EntitySlept { id, .. }
            | SimulationEvent::EntityDied { id, .. } => *id == FOCAL_ID,
            SimulationEvent::EntityAttacked {
                attacker, target, ..
            } => *attacker == FOCAL_ID || *target == FOCAL_ID,
            SimulationEvent::ActionRejected { id, .. } => *id == FOCAL_ID,
            SimulationEvent::CorpseConsumed { eater, .. } => *eater == FOCAL_ID,
            _ => false,
        })
        .map(|event| serde_json::to_value(event).expect("serialize focal event once"))
        .collect()
}

fn table(world: &World) -> Vec<GeneralizedRefugeOutcome> {
    world
        .entities
        .iter()
        .find(|slot| slot.id == FOCAL_ID)
        .map(|slot| slot.memory.refuge_generalized_outcomes.clone())
        .unwrap_or_default()
}

struct UpdateAudit {
    accepted_updates: usize,
    replay_matches_table: bool,
}

fn update_provenance(
    provenance: &mut Provenance,
    before: &[GeneralizedRefugeOutcome],
    after: &[GeneralizedRefugeOutcome],
    events: &[SimulationEvent],
    tick: usize,
    memory_observed: bool,
    generalization_enabled: bool,
    update_enabled: bool,
) -> UpdateAudit {
    let mut replay = before.to_vec();
    if memory_observed && generalization_enabled && update_enabled {
        retain_fresh_rows(&mut replay, tick);
    }
    let mut proposed_provenance = provenance.clone();
    proposed_provenance.retain_table_keys(&replay);
    let mut accepted_updates = 0;
    if update_enabled {
        for event in events {
            let SimulationEvent::RefugeEpisodeEnded {
                id,
                threat_id,
                ended_tick,
                closure,
                completed,
                attribution_known,
                threat_damage,
                movement_energy,
                generalized_context: Some(key),
                generalization_mode: Some(curiosity_mode),
                ..
            } = event
            else {
                continue;
            };
            if *id != FOCAL_ID
                || !*completed
                || !*attribution_known
                || *closure != curious::entity::memory::RefugeClosure::Arrived
                || *curiosity_mode != curious::entity::memory::RefugeLearningMode::Generalized
            {
                continue;
            }
            if curious::entity::refuge_generalization::record_completed_outcome(
                &mut replay,
                key.clone(),
                *ended_tick,
                *threat_damage,
                *movement_energy,
            ) {
                proposed_provenance.record(key.clone(), *threat_id, *ended_tick);
                accepted_updates += 1;
            }
        }
    }
    let replay_matches_table = replay == after;
    if replay_matches_table {
        *provenance = proposed_provenance;
    } else {
        provenance.reset();
        provenance.retain_table_keys(after);
        accepted_updates = 0;
    }
    UpdateAudit {
        accepted_updates,
        replay_matches_table,
    }
}

fn tick_once(
    world: &mut World,
    arm: &'static str,
    scope: &'static str,
    suffix_tick: Option<usize>,
    physical_tick_ordinal: usize,
    resolver_updates_before_tick: usize,
    provenance: &mut Provenance,
    trained_table_at_fork: Option<&[GeneralizedRefugeOutcome]>,
) -> (TickRecord, usize) {
    let checkpoint_before = State::capture(world);
    let state_hash_before = fnv1a64(&checkpoint_before);
    let rng_before = world.rng.state();
    let focal_before = focal_state(world);
    let pre_table = table(world);
    provenance.retain_table_keys(&pre_table);
    let pre_episode = world
        .entities
        .iter()
        .find(|slot| slot.id == FOCAL_ID)
        .and_then(|slot| slot.memory.refuge_episode.clone());
    let (generalization_enabled, update_enabled) = world
        .entities
        .iter()
        .find(|slot| slot.id == FOCAL_ID)
        .map(|slot| {
            (
                slot.memory.refuge_generalization_enabled,
                slot.memory.refuge_generalization_update_enabled,
            )
        })
        .unwrap_or((false, false));

    let events = world.tick();
    let mut choice = world
        .last_decisions
        .iter()
        .find(|trace| trace.id == FOCAL_ID)
        .map(|trace| {
            capture_choice(
                trace,
                &pre_table,
                pre_episode,
                provenance,
                generalization_enabled,
            )
        });
    if let Some(choice) = &mut choice {
        choice.table_differs_from_fork_at_choice =
            trained_table_at_fork.map(|fork_table| fork_table != pre_table.as_slice());
        choice.resolver_updates_before_choice = resolver_updates_before_tick;
    }
    let post_table = table(world);
    let update_audit = update_provenance(
        provenance,
        &pre_table,
        &post_table,
        &events,
        world.tick_counter,
        choice.is_some(),
        generalization_enabled,
        update_enabled,
    );
    let checkpoint_after = State::capture(world);
    let state_hash_after = fnv1a64(&checkpoint_after);
    let table_differs_from_fork =
        trained_table_at_fork.map(|fork_table| fork_table != post_table.as_slice());
    let row = TickRecord {
        scope,
        arm,
        suffix_tick,
        world_tick: world.tick_counter,
        physical_tick_ordinal,
        rng_state_before_world_tick: rng_before,
        rng_state_after_world_tick: world.rng.state(),
        world_state_fnv1a64_before: state_hash_before,
        world_state_fnv1a64_after: state_hash_after,
        full_pre_world_state_for_choice: choice
            .as_ref()
            .map(|_| serde_json::to_value(&checkpoint_before).expect("serialize choice pre-state")),
        focal_before,
        production_choice: choice,
        focal_events: focal_events(&events),
        focal_after: focal_state(world),
        table_changed_this_tick: pre_table != post_table,
        resolver_updates_this_tick: update_audit.accepted_updates,
        provenance_replay_matches_table: update_audit.replay_matches_table,
        trained_table_differs_from_fork: table_differs_from_fork,
    };
    (row, update_audit.accepted_updates)
}

fn make_fixture() -> World {
    let scenario: Scenario = serde_json::from_str(include_str!("../scenarios/mixed-refuge.json"))
        .expect("fixed checked-in mixed-refuge fixture parses");
    let mut world = scenario.build(ENGINEERING_SEED).expect("fixture builds");
    world.logger = Logger::disabled();
    let selected_focal = select_focal_id(&world).expect("fixture has a herbivore and predator");
    assert_eq!(
        selected_focal, FOCAL_ID,
        "predeclared focal-selection rule changed"
    );
    world.observation.full_for = vec![selected_focal];
    let focal = world
        .entities
        .iter_mut()
        .find(|slot| slot.id == selected_focal)
        .expect("fixture focal founder exists");
    assert_eq!(focal.base.species(), Species::Herbivore);
    focal.memory.refuge_generalization_enabled = true;
    // Default is true; assign explicitly as the one permitted learning setup.
    focal.memory.refuge_generalization_update_enabled = true;
    assert!(focal.memory.refuge_generalized_outcomes.is_empty());
    assert!(focal.memory.refuge_episode.is_none());
    world
}

/// Select from unchanged tick-zero founder bodies by observable position and
/// species. This affects only which individual is instrumented and enabled.
fn select_focal_id(world: &World) -> Option<usize> {
    let herbivore = Species::Herbivore;
    let predator_positions = world
        .entities
        .iter()
        .filter(|slot| slot.base.species().hunts(&herbivore))
        .map(|slot| slot.pos)
        .collect::<Vec<_>>();
    world
        .entities
        .iter()
        .filter(|slot| slot.base.species() == Species::Herbivore)
        .filter_map(|herbivore_slot| {
            predator_positions
                .iter()
                .map(|position| herbivore_slot.pos.distance_to(*position))
                .min()
                .map(|distance| (distance, herbivore_slot.id))
        })
        .min()
        .map(|(_, id)| id)
}

fn expected_treatment_state(base: &State, treatment: Treatment) -> State {
    let mut expected = base.clone();
    if let Some(focal) = expected
        .entities
        .iter_mut()
        .find(|individual| individual.id == FOCAL_ID)
    {
        match treatment {
            Treatment::TrainedOnline => {}
            Treatment::TrainedFrozen => {
                focal.memory.refuge_generalization_update_enabled = false;
            }
            Treatment::NaiveOnline => {
                focal.memory.refuge_generalized_outcomes.clear();
            }
            Treatment::RuleBased => {
                focal.memory.refuge_generalization_enabled = false;
            }
        }
    }
    expected
}

fn treatment_difference(treatment: Treatment) -> &'static str {
    match treatment {
        Treatment::TrainedOnline => "none; exact prefix state retained",
        Treatment::TrainedFrozen => "update_enabled=false only",
        Treatment::NaiveOnline => "generalized_outcomes cleared only; active episode retained",
        Treatment::RuleBased => "generalization_enabled=false only; active episode retained",
    }
}

fn arm_run<W: Write>(
    writer: &mut W,
    base_state: &State,
    treatment: Treatment,
    prefix_provenance: &Provenance,
    physical_offset: usize,
) -> io::Result<(usize, Vec<ObservedChoice>, bool)> {
    let mut branch = base_state.clone().restore().expect("restore forked prefix");
    let expected = expected_treatment_state(base_state, treatment);
    if let Some(focal) = branch.entities.iter_mut().find(|slot| slot.id == FOCAL_ID) {
        match treatment {
            Treatment::TrainedOnline => {}
            Treatment::TrainedFrozen => {
                focal.memory.refuge_generalization_update_enabled = false;
            }
            Treatment::NaiveOnline => {
                focal.memory.refuge_generalized_outcomes.clear();
            }
            Treatment::RuleBased => {
                focal.memory.refuge_generalization_enabled = false;
            }
        }
    }
    let actual_start = State::capture(&branch);
    assert_eq!(
        serde_json::to_value(&actual_start).unwrap(),
        serde_json::to_value(&expected).unwrap(),
        "arm must differ from the shared prefix only by its declared treatment"
    );
    let inherited_active_episode = branch
        .entities
        .iter()
        .find(|slot| slot.id == FOCAL_ID)
        .and_then(|slot| slot.memory.refuge_episode.clone());
    emit(
        writer,
        &RawRecord::TreatmentStart {
            arm: treatment.label(),
            focal_present: branch.entities.iter().any(|slot| slot.id == FOCAL_ID),
            state: serde_json::to_value(&actual_start).unwrap(),
            expected_only_treatment_difference: treatment_difference(treatment),
            inherited_active_episode,
        },
    )?;

    let fork_table = table(&branch);
    let mut provenance = prefix_provenance.clone();
    if matches!(treatment, Treatment::NaiveOnline) {
        provenance.reset();
    }
    provenance.retain_table_keys(&fork_table);
    let mut update_count = 0;
    let mut choices = Vec::new();
    let retain_choices_for_pair = matches!(
        treatment,
        Treatment::TrainedOnline | Treatment::TrainedFrozen
    );
    for suffix_tick in 1..=SUFFIX_TICKS {
        let (row, updates) = tick_once(
            &mut branch,
            treatment.label(),
            "suffix",
            Some(suffix_tick),
            physical_offset + suffix_tick,
            update_count,
            &mut provenance,
            Some(&fork_table),
        );
        update_count += updates;
        if retain_choices_for_pair {
            if let (Some(choice), Some(full_pre_world_state)) = (
                row.production_choice.clone(),
                row.full_pre_world_state_for_choice.as_ref(),
            ) {
                choices.push(ObservedChoice {
                    suffix_tick,
                    witness: choice,
                    normalized_pre_world_state: normalized_pre_world_state_bytes(
                        full_pre_world_state.clone(),
                        FOCAL_ID,
                    ),
                    focal_events: row.focal_events.clone(),
                });
            }
        }
        emit(writer, &RawRecord::SuffixTick(row))?;
    }
    let end_table = table(&branch);
    let table_mutated = end_table != fork_table;
    emit(
        writer,
        &RawRecord::BranchEnd {
            arm: treatment.label(),
            physical_ticks: SUFFIX_TICKS,
            focal_terminal_state: focal_state(&branch),
            table_at_fork: fork_table,
            table_at_end: end_table,
            provenance_at_end: provenance.rows().to_vec(),
            table_mutated,
        },
    )?;
    Ok((update_count, choices, table_mutated))
}

fn choice_is_supported_fresh_greedy(choice: &ChoiceWitness) -> bool {
    choice.is_fresh
        && choice.selection_kind == "generalized_supported_greedy"
        && choice.selected_candidate_index.is_some()
        && !choice.candidates.is_empty()
        && choice
            .candidates
            .iter()
            .all(|candidate| candidate.supported_unique_fresh)
}

fn same_choice_context(left: &ObservedChoice, right: &ObservedChoice) -> bool {
    left.suffix_tick == right.suffix_tick
        && left.normalized_pre_world_state == right.normalized_pre_world_state
        && left.witness.choice_tick == right.witness.choice_tick
        && left.witness.position == right.witness.position
        && serde_json::to_value(&left.witness.life_before_choice).ok()
            == serde_json::to_value(&right.witness.life_before_choice).ok()
        && left.witness.rng_before_choice == right.witness.rng_before_choice
        && left.witness.threat_id == right.witness.threat_id
        && serde_json::to_value(&left.witness.perception_witness).ok()
            == serde_json::to_value(&right.witness.perception_witness).ok()
        && left.witness.candidates.len() == right.witness.candidates.len()
        && left
            .witness
            .candidates
            .iter()
            .zip(&right.witness.candidates)
            .all(|(a, b)| a.route == b.route && a.endpoint == b.endpoint && a.key == b.key)
        && serde_json::to_value(&left.witness.active_episode_before).ok()
            == serde_json::to_value(&right.witness.active_episode_before).ok()
}

fn focal_physical_events(events: &[Value]) -> Vec<&Value> {
    events
        .iter()
        .filter(|event| event.get("EntityMoved").is_some() || event.get("EntityAttacked").is_some())
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut out = CappedWriter {
        inner: io::BufWriter::new(io::stdout().lock()),
        written: 0,
        limit: 128 * 1024 * 1024,
    };
    emit(
        &mut out,
        &RawRecord::Header {
            protocol: PROTOCOL,
            scenario: "scenarios/mixed-refuge.json",
            seed: ENGINEERING_SEED,
            focal_id: FOCAL_ID,
            prefix_ticks: PREFIX_TICKS,
            suffix_ticks_per_arm: SUFFIX_TICKS,
            max_physical_ticks: MAX_PHYSICAL_TICKS,
            state_hash_algorithm: "FNV-1a 64-bit over serde_json(State); witness only",
            evidence_role: "engineering feasibility only; seed 123 is seen, not formal",
        },
    )?;

    let mut prefix = make_fixture();
    let initial_state = State::capture(&prefix);
    emit(
        &mut out,
        &RawRecord::InitialState {
            state: serde_json::to_value(&initial_state)?,
            rng_state: initial_state.rng,
        },
    )?;
    let mut prefix_provenance = Provenance::default();
    let mut prefix_updates = 0;
    for tick in 1..=PREFIX_TICKS {
        let (row, updates) = tick_once(
            &mut prefix,
            "shared_prefix_online",
            "prefix",
            None,
            tick,
            prefix_updates,
            &mut prefix_provenance,
            None,
        );
        prefix_updates += updates;
        emit(&mut out, &RawRecord::PrefixTick(row))?;
    }
    let prefix_state = State::capture(&prefix);
    emit(
        &mut out,
        &RawRecord::Fork {
            state: serde_json::to_value(&prefix_state)?,
            rng_state: prefix_state.rng,
            prefix_physical_ticks: PREFIX_TICKS,
            provenance: prefix_provenance.rows().to_vec(),
        },
    )?;
    let mut total_updates = prefix_updates;
    let mut all_choices = std::collections::BTreeMap::<&'static str, Vec<ObservedChoice>>::new();
    let mut branch_table_mutated = std::collections::BTreeMap::<&'static str, bool>::new();
    let mut physical_offset = PREFIX_TICKS;
    for treatment in Treatment::ALL {
        let (updates, choices, table_mutated) = arm_run(
            &mut out,
            &prefix_state,
            treatment,
            &prefix_provenance,
            physical_offset,
        )?;
        total_updates += updates;
        if matches!(
            treatment,
            Treatment::TrainedOnline | Treatment::TrainedFrozen
        ) {
            all_choices.insert(treatment.label(), choices);
        }
        branch_table_mutated.insert(treatment.label(), table_mutated);
        physical_offset += SUFFIX_TICKS;
    }
    assert_eq!(physical_offset, MAX_PHYSICAL_TICKS);

    let trained = all_choices.get("trained_online").unwrap();
    let frozen = all_choices.get("trained_frozen").unwrap();
    let mut supported_fresh = 0;
    let mut different_id_support = 0;
    let mut contrasts = 0;
    for observation in trained {
        let choice = &observation.witness;
        if choice_is_supported_fresh_greedy(choice) {
            supported_fresh += 1;
            if choice.threat_id.is_some_and(|id| {
                choice.candidates.iter().all(|candidate| {
                    candidate.supported_unique_fresh
                        && candidate.different_threat_id_supported
                        && candidate.support_sources.iter().any(|source| *source != id)
                })
            }) {
                different_id_support += 1;
            }
        }
    }
    let frozen_by_tick = frozen
        .iter()
        .map(|observation| (observation.suffix_tick, observation))
        .collect::<std::collections::BTreeMap<_, _>>();
    for online_observation in trained {
        let online = &online_observation.witness;
        let Some(frozen_observation) = frozen_by_tick.get(&online_observation.suffix_tick) else {
            continue;
        };
        let frozen = &frozen_observation.witness;
        if !same_choice_context(online_observation, frozen_observation)
            || !choice_is_supported_fresh_greedy(online)
            || !choice_is_supported_fresh_greedy(frozen)
        {
            continue;
        }
        let routes_differ = online.selected_candidate_index != frozen.selected_candidate_index;
        let physical_events_differ = focal_physical_events(&online_observation.focal_events)
            != focal_physical_events(&frozen_observation.focal_events);
        let trained_changed = online.resolver_updates_before_choice > 0
            && online.table_differs_from_fork_at_choice == Some(true);
        if routes_differ && physical_events_differ && trained_changed {
            contrasts += 1;
        }
    }
    let all_gates_pass = prefix_updates > 0
        && supported_fresh > 0
        && different_id_support > 0
        && contrasts > 0
        && branch_table_mutated.get("trained_frozen") == Some(&false);
    emit(
        &mut out,
        &RawRecord::Summary {
            physical_ticks: physical_offset,
            distinct_world_ticks: PREFIX_TICKS + SUFFIX_TICKS,
            focal_update_events: total_updates,
            post_prefix_supported_fresh_greedy_choices: supported_fresh,
            different_threat_id_supported_choices: different_id_support,
            aligned_online_frozen_greedy_contrasts: contrasts,
            opportunity_gate: if all_gates_pass {
                "engineering_opportunity_present"
            } else {
                "engineering_opportunity_not_identified"
            },
            interpretation: "one fixed seen-seed fixture only; no formal or utility claim",
        },
    )?;
    Ok(())
}
