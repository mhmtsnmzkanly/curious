//! Doc-hidden, feature-gated single-tick resolver ordering probe.
use super::{ActionPlans, EatPlan, PreparedTick, World};
use crate::{
    entity::{
        decision::{DecisionTrace, Target, WaterMealOverride},
        intent::Intent,
        learning::WaterEnd,
    },
    simulation::{SimulationEvent, checkpoint::State},
    systems::planning::PlanningSystem,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Machine-readable output for one bounded resolver-order case.
#[derive(Debug)]
pub struct ResolverOrderLabCase(pub Value);

impl World {
    /// Run three resolver variants from one ordinary off-policy preparation.
    /// This is only compiled when the `resolver-lab` feature is enabled.
    #[doc(hidden)]
    pub fn resolver_order_lab_tick(&mut self) -> crate::Result<ResolverOrderLabCase> {
        self.resolver_order_lab_tick_checked(|_| {
            Ok(json!({"valid":true,"validation":"not_supplied"}))
        })
    }

    /// Validate the already-computed native-off target tick before preparing A/B/C.
    #[doc(hidden)]
    pub fn resolver_order_lab_tick_checked<F>(
        &mut self,
        validate_native_off: F,
    ) -> crate::Result<ResolverOrderLabCase>
    where
        F: FnOnce(&Value) -> crate::Result<Value>,
    {
        if self.policy.water_meal_priority_enabled {
            return Err("resolver lab requires a water-meal-off pre-tick state".into());
        }
        let pre_state = State::capture(self);
        let pre_target_roundtrip = state_roundtrip_valid(&pre_state)?;
        if !pre_target_roundtrip {
            return Err("pre-target State/RNG restore round-trip failed".into());
        }
        let pre_decisions = self.last_decisions.clone();
        let pre_intents = self.last_intents.clone();
        let pre_trace = self.last_trace.clone();

        // The independent references use the exact same pre-tick State and transient decision
        // context. The policy flag is the only input changed for the native-on reference.
        let native_off =
            run_reference(&pre_state, &pre_decisions, &pre_intents, &pre_trace, false)?;
        let target_validation = match validate_native_off(&native_off) {
            Ok(value) => value,
            Err(error) => {
                return Ok(ResolverOrderLabCase(json!({
                    "schema":"curious-resolver-order-v1-case",
                    "status":"failed",
                    "integrity_failures":["native-off reference did not match raw target tick"],
                    "target_validation":{"valid":false,"error":error.to_string()},
                    "references":{"native_off":native_off}
                })));
            }
        };
        let native_on = run_reference(&pre_state, &pre_decisions, &pre_intents, &pre_trace, true)?;

        let mut perceptions = Vec::new();
        let prepared = self.prepare_tick(Some(&mut perceptions));
        let base_unrotated = prepared.plans.clone();
        let mut base_rotated = base_unrotated.clone();
        base_rotated.rotate_priority(self.tick_counter);

        let (candidate_intents, candidate_traces, overrides, interrupt_ids) =
            derive_candidate(self, &perceptions)?;
        let mut planning_log = Vec::new();
        let candidate_unrotated =
            PlanningSystem::plan(self, candidate_intents.clone(), &mut planning_log);
        let mut candidate_rotated = candidate_unrotated.clone();
        candidate_rotated.rotate_priority(self.tick_counter);
        let candidate_fixed =
            fixed_candidate_order(&base_rotated, &candidate_rotated, self.tick_counter);

        let common_state = State::capture(self);
        let common_state_value = serde_json::to_value(&common_state)?;
        let common_rng = self.rng.state();
        let common_decisions = self.last_decisions.clone();
        let common_intents = self.last_intents.clone();
        let common_trace = self.last_trace.clone();
        let branch_roundtrip_valid = state_roundtrip_valid(&common_state)?;

        let mut branch_a_world = restore_with_transients(
            common_state.clone(),
            &common_decisions,
            &common_intents,
            &common_trace,
        )?;
        let a_roundtrip = state_roundtrip_valid(&State::capture(&branch_a_world))?;
        let a = run_branch(
            &mut branch_a_world,
            prepared.clone(),
            base_rotated.clone(),
            common_decisions.clone(),
            common_intents.clone(),
        )?;

        let mut branch_b_world = restore_with_transients(
            common_state.clone(),
            &candidate_traces,
            &candidate_intents,
            &common_trace,
        )?;
        apply_candidate_interruptions(&mut branch_b_world, &interrupt_ids);
        let b_pre_state = State::capture(&branch_b_world);
        let b_pre_decisions = branch_b_world.last_decisions.clone();
        let b_pre_intents = branch_b_world.last_intents.clone();
        let b_roundtrip = state_roundtrip_valid(&b_pre_state)?;
        let candidate_fixed_audit = candidate_fixed.clone();
        let b = run_branch(
            &mut branch_b_world,
            prepared.clone(),
            candidate_fixed,
            candidate_traces.clone(),
            candidate_intents.clone(),
        )?;

        let mut branch_c_world = restore_with_transients(
            common_state.clone(),
            &candidate_traces,
            &candidate_intents,
            &common_trace,
        )?;
        apply_candidate_interruptions(&mut branch_c_world, &interrupt_ids);
        let c_pre_state = State::capture(&branch_c_world);
        let c_pre_decisions = branch_c_world.last_decisions.clone();
        let c_pre_intents = branch_c_world.last_intents.clone();
        let c_roundtrip = state_roundtrip_valid(&c_pre_state)?;
        let candidate_rotated_audit = candidate_rotated.clone();
        let c = run_branch(
            &mut branch_c_world,
            prepared,
            candidate_rotated,
            candidate_traces.clone(),
            candidate_intents.clone(),
        )?;

        let b_pre_value = serde_json::to_value(&b_pre_state)?;
        let c_pre_value = serde_json::to_value(&c_pre_state)?;
        let candidate_pre_audit = json!({
            "equal": b_pre_value == c_pre_value,
            "learner_interruptions_equal": state_learners(&b_pre_value, &interrupt_ids) == state_learners(&c_pre_value, &interrupt_ids),
            "decision_trace_equal": serde_json::to_value(&b_pre_decisions)? == serde_json::to_value(&c_pre_decisions)?,
            "intent_equal": serde_json::to_value(&b_pre_intents)? == serde_json::to_value(&c_pre_intents)?,
            "plan_content_equal": plan_content(&candidate_fixed_audit) == plan_content(&candidate_rotated_audit),
        });
        let candidate_pre_equal = candidate_pre_audit["equal"] == true
            && candidate_pre_audit["learner_interruptions_equal"] == true
            && candidate_pre_audit["decision_trace_equal"] == true
            && candidate_pre_audit["intent_equal"] == true
            && candidate_pre_audit["plan_content_equal"] == true;
        let all_branch_roundtrips_valid =
            branch_roundtrip_valid && a_roundtrip && b_roundtrip && c_roundtrip;

        let a_matches_native_off = branch_matches_reference(&a, &native_off, false)?;
        let c_matches_native_on = branch_matches_reference(&c, &native_on, true)?;
        let mut integrity_failures = Vec::new();
        if !all_branch_roundtrips_valid {
            integrity_failures.push("branch State/RNG restore round-trip failed");
        }
        if !candidate_pre_equal {
            integrity_failures.push("candidate B/C pre-resolution audit differs");
        }
        if !a_matches_native_off {
            integrity_failures.push("A branch did not match ordinary native water-meal-off tick");
        }
        if !c_matches_native_on {
            integrity_failures.push("C branch did not match ordinary native water-meal-on tick");
        }

        let value = json!({
            "schema":"curious-resolver-order-v1-case",
            "status":if integrity_failures.is_empty() {"complete"} else {"failed"},
            "integrity_failures":integrity_failures,
            "pre_target_roundtrip":{"valid":pre_target_roundtrip,"state_equal":pre_target_roundtrip,"rng_equal":pre_target_roundtrip,
                "state":serde_json::to_value(&pre_state)?,"rng":pre_state.rng},
            "branch_roundtrip_valid":all_branch_roundtrips_valid,
            "target_validation":target_validation,
            "candidate_b_equals_c_pre_resolution_audit":candidate_pre_audit,
            "references":{
                "a_matches_native_off":a_matches_native_off,
                "c_matches_native_on":c_matches_native_on,
                "compared_fields":["events","rng","last_intents","last_decisions","state"],
                "normalizations":["policy.water_meal_priority_enabled only"],
                "native_off":native_off,
                "native_on":native_on
            },
            "common_prepared":{
                "state":common_state_value,"rng":common_rng,
                "actor_inputs":actor_inputs(self, &perceptions)?,
                "baseline_intents":serde_json::to_value(&common_intents)?,
                "baseline_plans":{"unrotated":plans_value(&base_unrotated),"rotated":plans_value(&base_rotated)},
                "candidate_intents":serde_json::to_value(&candidate_intents)?,
                "candidate_plans":{"unrotated":plans_value(&candidate_unrotated),"fixed_B":plans_value(&candidate_fixed_audit),"rotated_C":plans_value(&candidate_rotated_audit)},
                "candidate_overrides":overrides
            },
            "branches":{"A":a,"B":b,"C":c}
        });
        Ok(ResolverOrderLabCase(value))
    }
}

fn run_reference(
    state: &State,
    decisions: &[DecisionTrace],
    intents: &[(usize, Intent)],
    trace: &[String],
    water_meal: bool,
) -> crate::Result<Value> {
    let mut world = restore_with_transients(state.clone(), decisions, intents, trace)?;
    world.policy.water_meal_priority_enabled = water_meal;
    let events = world.tick();
    Ok(json!({
        "water_meal_priority_enabled":water_meal,
        "events":serde_json::to_value(events)?,
        "rng":world.rng.state(),
        "last_intents":serde_json::to_value(&world.last_intents)?,
        "last_decisions":serde_json::to_value(&world.last_decisions)?,
        "state":serde_json::to_value(State::capture(&world))?
    }))
}

fn restore_with_transients(
    state: State,
    decisions: &[DecisionTrace],
    intents: &[(usize, Intent)],
    trace: &[String],
) -> crate::Result<World> {
    let mut world = state.restore()?;
    world.last_decisions = decisions.to_vec();
    world.last_intents = intents.to_vec();
    world.last_trace = trace.to_vec();
    Ok(world)
}

fn state_roundtrip_valid(state: &State) -> crate::Result<bool> {
    let before = serde_json::to_value(state)?;
    let rng = state.rng;
    let restored = state.clone().restore()?;
    Ok(serde_json::to_value(State::capture(&restored))? == before && restored.rng.state() == rng)
}

fn state_learners(state: &Value, ids: &[usize]) -> Value {
    let wanted: BTreeSet<_> = ids.iter().copied().collect();
    Value::Array(
        state
            .get("entities")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|entity| {
                entity
                    .get("id")
                    .and_then(Value::as_u64)
                    .is_some_and(|id| wanted.contains(&(id as usize)))
            })
            .map(|entity| json!({"id":entity.get("id"),"learner":entity.get("learner")}))
            .collect(),
    )
}

fn derive_candidate(
    world: &World,
    perceptions: &[(usize, crate::entity::perception::Perception)],
) -> crate::Result<(Vec<(usize, Intent)>, Vec<DecisionTrace>, Value, Vec<usize>)> {
    let views: HashMap<_, _> = perceptions.iter().map(|(id, view)| (*id, view)).collect();
    let mut decisions = world.last_decisions.clone();
    let mut intents = world.last_intents.clone();
    let mut rows = Vec::new();
    let mut interrupt_ids = Vec::new();
    for trace in &mut decisions {
        let Some(view) = views.get(&trace.id) else {
            continue;
        };
        let Some(meal) = crate::policy::prefer_adjacent_meal_before_unpayable_water(
            trace.species.clone(),
            &trace.before,
            view,
            trace.decision.clone(),
            world.policy.injury_capacity_penalty,
        ) else {
            continue;
        };
        let replaced_decision = trace.decision.clone();
        let water_amount = match &replaced_decision.intent {
            Intent::Drink { at } => view
                .waters
                .iter()
                .find(|w| w.amount > 0 && w.steps == *at)
                .map(|w| w.amount)
                .unwrap_or(0),
            _ => 0,
        };
        let food_amount = match &meal.intent {
            Intent::Eat {
                at,
                corpse_id: None,
            } => view
                .foods
                .iter()
                .find(|f| !f.is_corpse && f.corpse_id.is_none() && f.steps == *at)
                .map(|f| f.amount)
                .unwrap_or(0),
            _ => 0,
        };
        if water_amount == 0 || food_amount == 0 {
            return Err(format!(
                "actor {} override lacked visible positive resource amounts",
                trace.id
            )
            .into());
        }
        trace.water_meal_override = Some(WaterMealOverride {
            replaced_decision: replaced_decision.clone(),
            visible_water_amount: water_amount,
            visible_plant_food_amount: food_amount,
        });
        trace.decision = meal.clone();
        trace.target = match &meal.intent {
            Intent::Move { steps }
            | Intent::Eat { at: steps, .. }
            | Intent::Drink { at: steps } => Some(Target::Position(
                steps.0.iter().fold(trace.pos, |p, d| p + *d),
            )),
            Intent::Attack { target_id }
            | Intent::Flee { target_id }
            | Intent::Mate { target_id } => Some(Target::Entity(*target_id)),
            _ => None,
        };
        if let Some((_, intent)) = intents.iter_mut().find(|(id, _)| *id == trace.id) {
            *intent = meal.intent.clone();
        } else {
            return Err(format!(
                "override actor {} missing from baseline intent list",
                trace.id
            )
            .into());
        }
        interrupt_ids.push(trace.id);
        rows.push(
            json!({"id":trace.id,"before_decision":replaced_decision,"candidate_decision":meal,
            "visible_water_amount":water_amount,"visible_plant_food_amount":food_amount}),
        );
    }
    Ok((intents, decisions, Value::Array(rows), interrupt_ids))
}

fn apply_candidate_interruptions(world: &mut World, ids: &[usize]) {
    for slot in &mut world.entities {
        if ids.contains(&slot.id) {
            slot.learner.interrupt(WaterEnd::Interrupted);
        }
    }
}

fn run_branch(
    world: &mut World,
    mut prepared: PreparedTick,
    plans: ActionPlans,
    decisions: Vec<DecisionTrace>,
    intents: Vec<(usize, Intent)>,
) -> crate::Result<Value> {
    let plan_orders = plans_value(&plans);
    world.last_decisions = decisions;
    world.last_intents = intents;
    let pre_state = State::capture(world);
    let pre_state_value = serde_json::to_value(&pre_state)?;
    let pre_rng = world.rng.state();
    let pre_bodies = actor_bodies(world);
    prepared.plans = plans.clone();
    let events = world.resolve_prepared_tick(prepared, plans);
    Ok(json!({
        "pre_state":pre_state_value,"pre_rng":pre_rng,"pre_actor_bodies":pre_bodies,
        "plan_orders":plan_orders,
        "events":serde_json::to_value(&events)?,"event_totals":event_totals(&events),
        "post_state":serde_json::to_value(State::capture(world))?,"post_rng":world.rng.state(),
        "post_actor_bodies":actor_bodies(world),
        "post_last_intents":serde_json::to_value(&world.last_intents)?,
        "post_last_decisions":serde_json::to_value(&world.last_decisions)?,
        "focal_decision_trace":world.last_decisions.iter().find(|trace| trace.id == 1)
            .map(serde_json::to_value).transpose()?.unwrap_or(Value::Null)
    }))
}

fn actor_inputs(
    world: &World,
    perceptions: &[(usize, crate::entity::perception::Perception)],
) -> crate::Result<Value> {
    let views: HashMap<_, _> = perceptions.iter().map(|(id, view)| (*id, view)).collect();
    let mut rows = Vec::new();
    for slot in &world.entities {
        let decision = world.last_decisions.iter().find(|d| d.id == slot.id);
        rows.push(
            json!({"id":slot.id,"pos":slot.pos,"phase":slot.phase,"body":slot.base.life(),
            "perception":views.get(&slot.id).copied(),"decision_trace":decision}),
        );
    }
    Ok(Value::Array(rows))
}

fn actor_bodies(world: &World) -> Value {
    Value::Array(world.entities.iter().map(|slot| json!({"id":slot.id,"pos":slot.pos,"phase":slot.phase,"life":slot.base.life()})).collect())
}

fn plans_value(plans: &ActionPlans) -> Value {
    json!({
        "move":plans.moves.iter().map(|p| json!({"id":p.id,"path":p.path,"new_pos":p.new_pos,"cost":p.cost})).collect::<Vec<_>>(),
        "eat":plans.eats.iter().map(eat_value).collect::<Vec<_>>(),
        "drink":plans.drinks.iter().map(|p| json!({"id":p.id,"path":p.path,"new_pos":p.new_pos,"cost":p.cost,"resource_pos":p.resource_pos})).collect::<Vec<_>>(),
        "mate":plans.mates.iter().map(|p| json!({"id":p.self_id,"target_id":p.target_id})).collect::<Vec<_>>(),
        "attack":plans.attacks.iter().map(|p| json!({"id":p.attacker_id,"target_id":p.target_id})).collect::<Vec<_>>(),
        "flee":plans.flees.iter().map(|p| json!({"id":p.id,"path":p.path,"new_pos":p.new_pos,"cost":p.cost})).collect::<Vec<_>>(),
        "sleep":plans.sleeps.iter().map(|p| json!({"id":p.id,"duration":p.duration})).collect::<Vec<_>>(),
    })
}
fn eat_value(p: &EatPlan) -> Value {
    json!({"id":p.id,"path":p.path,"new_pos":p.new_pos,"cost":p.cost,"resource_pos":p.resource_pos,"corpse_id":p.corpse_id})
}

fn fixed_candidate_order(
    base: &ActionPlans,
    candidate_rotated: &ActionPlans,
    tick: usize,
) -> ActionPlans {
    let mut out = candidate_rotated.clone();
    let candidate_eats: HashMap<_, _> = candidate_rotated
        .eats
        .iter()
        .map(|p| (p.id, p.clone()))
        .collect();
    let candidate_ids: BTreeSet<_> = candidate_eats.keys().copied().collect();
    let mut eats = Vec::new();
    for p in &base.eats {
        if candidate_ids.contains(&p.id) {
            eats.push(candidate_eats[&p.id].clone());
        }
    }
    let mut new: Vec<_> = candidate_rotated
        .eats
        .iter()
        .filter(|p| !base.eats.iter().any(|b| b.id == p.id))
        .cloned()
        .collect();
    new.sort_by_key(|p| p.id);
    eats.extend(new);
    out.eats = eats;
    let drink_ids: BTreeSet<_> = candidate_rotated.drinks.iter().map(|p| p.id).collect();
    out.drinks = base
        .drinks
        .iter()
        .filter(|p| drink_ids.contains(&p.id))
        .cloned()
        .collect();
    let _ = tick;
    out
}

fn plan_content(plans: &ActionPlans) -> Value {
    let mut value = plans_value(plans);
    if let Some(object) = value.as_object_mut() {
        for list in object.values_mut() {
            if let Some(rows) = list.as_array_mut() {
                rows.sort_by_key(|row| {
                    row.get("id")
                        .or_else(|| row.get("target_id"))
                        .cloned()
                        .unwrap_or(Value::Null)
                        .to_string()
                });
            }
        }
    }
    value
}

fn event_totals(events: &[SimulationEvent]) -> Value {
    let mut food = 0usize;
    let mut water = 0usize;
    let mut movement = 0usize;
    let mut rejected_eat = 0usize;
    let mut rejected_drink = 0usize;
    let mut deaths = 0usize;
    let mut by_actor: BTreeMap<usize, Value> = BTreeMap::new();
    for event in events {
        let value = serde_json::to_value(event).unwrap_or(Value::Null);
        if let Some((name, payload)) = value.as_object().and_then(|o| o.iter().next()) {
            let actor = payload
                .get("id")
                .or_else(|| payload.get("attacker"))
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize;
            match name.as_str() {
                "EntityAte" if payload.get("corpse_id").unwrap_or(&Value::Null).is_null() => {
                    food += payload.get("amount").and_then(Value::as_u64).unwrap_or(0) as usize
                }
                "EntityDrank" => {
                    water += payload.get("amount").and_then(Value::as_u64).unwrap_or(0) as usize
                }
                "EntityMoved" => {
                    movement += payload
                        .get("energy_cost")
                        .and_then(Value::as_u64)
                        .unwrap_or(0) as usize
                }
                "ActionRejected" => match payload.get("action").and_then(Value::as_str) {
                    Some("Eat") => rejected_eat += 1,
                    Some("Drink") => rejected_drink += 1,
                    _ => {}
                },
                "EntityDied" => deaths += 1,
                _ => {}
            }
            let row = by_actor.entry(actor).or_insert_with(|| json!({"food":0,"water":0,"movement_energy":0,"attack_energy":0,"eat_rejections":0,"drink_rejections":0,"deaths":0}));
            match name.as_str() {
                "EntityAte" if payload.get("corpse_id").unwrap_or(&Value::Null).is_null() => {
                    increment(
                        row,
                        "food",
                        payload.get("amount").and_then(Value::as_u64).unwrap_or(0),
                    )
                }
                "EntityDrank" => increment(
                    row,
                    "water",
                    payload.get("amount").and_then(Value::as_u64).unwrap_or(0),
                ),
                "EntityMoved" => increment(
                    row,
                    "movement_energy",
                    payload
                        .get("energy_cost")
                        .and_then(Value::as_u64)
                        .unwrap_or(0),
                ),
                "EntityAttacked" => increment(
                    row,
                    "attack_energy",
                    payload
                        .get("energy_cost")
                        .and_then(Value::as_u64)
                        .unwrap_or(0),
                ),
                "ActionRejected" => match payload.get("action").and_then(Value::as_str) {
                    Some("Eat") => increment(row, "eat_rejections", 1),
                    Some("Drink") => increment(row, "drink_rejections", 1),
                    _ => {}
                },
                "EntityDied" => increment(row, "deaths", 1),
                _ => {}
            }
        }
    }
    json!({"plant_food_energy":food,"water":water,"movement_energy":movement,"eat_rejections":rejected_eat,"drink_rejections":rejected_drink,"deaths":deaths,"by_actor":by_actor})
}

fn increment(object: &mut Value, key: &str, amount: u64) {
    let old = object.get(key).and_then(Value::as_u64).unwrap_or(0);
    object[key] = Value::from(old.saturating_add(amount));
}

fn branch_matches_reference(
    branch: &Value,
    reference: &Value,
    normalize_flag: bool,
) -> crate::Result<bool> {
    let mut branch_state = branch
        .get("post_state")
        .cloned()
        .ok_or("branch missing state")?;
    let mut reference_state = reference
        .get("state")
        .cloned()
        .ok_or("reference missing state")?;
    if normalize_flag {
        branch_state["policy"]["water_meal_priority_enabled"] = Value::Bool(false);
        reference_state["policy"]["water_meal_priority_enabled"] = Value::Bool(false);
    }
    Ok(branch.get("events") == reference.get("events")
        && branch.get("post_rng") == reference.get("rng")
        && branch.get("post_last_intents") == reference.get("last_intents")
        && branch.get("post_last_decisions") == reference.get("last_decisions")
        && branch_state == reference_state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        creatures::herbivore::HerbivoreEntity,
        entity::{Entity, learning::Learner, phase::EntityPhase},
        map::cell::Cell,
        map::movement::Position,
        world::{DrinkPlan, EntitySlot, MovePlan},
    };

    fn eat(id: usize, x: isize) -> EatPlan {
        EatPlan {
            id,
            path: vec![],
            new_pos: Position::new(x, 0),
            cost: 0,
            resource_pos: Position::new(x, 0),
            corpse_id: None,
        }
    }

    #[test]
    fn empty_synthetic_world_matches_native_off_and_on_tick_references() {
        let mut world = World::with_seed(-2, 2, -2, 2, vec![], 17);
        let result = world.resolver_order_lab_tick().unwrap().0;
        assert_eq!(result["references"]["a_matches_native_off"], true);
        assert_eq!(result["references"]["c_matches_native_on"], true);
        assert_eq!(result["branch_roundtrip_valid"], true);
        assert_eq!(
            result["candidate_b_equals_c_pre_resolution_audit"]["equal"],
            true
        );
    }

    #[test]
    fn failed_native_off_target_gate_stops_before_shared_preparation_and_forks() {
        let mut world = World::with_seed(-2, 2, -2, 2, vec![], 19);
        let result = world
            .resolver_order_lab_tick_checked(|_| Err("synthetic raw target mismatch".into()))
            .unwrap()
            .0;
        assert_eq!(result["status"], "failed");
        assert!(result.pointer("/references/native_off/state").is_some());
        assert!(result.pointer("/references/native_on").is_none());
        assert!(result.get("branches").is_none());
        assert_eq!(
            world.tick_counter, 0,
            "failed target gate must precede shared preamble"
        );
    }

    #[test]
    fn adjacent_meal_override_round_trips_interrupt_and_resolves_identically_to_native_on() {
        let mut herbivore = HerbivoreEntity::default();
        let life = herbivore.life_mut();
        life.energy = 1;
        life.water = 1;
        life.age = 1;
        life.max_age = 2_000;
        life.maturity_age = 2_000;
        life.health = life.max_health;
        let slot = EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            crate::entity::controller::EntityController::AI,
            Box::new(herbivore),
        );
        let mut world = World::with_seed(-5, 5, -5, 5, vec![slot], 7);
        for y in world.map.min_y..=world.map.max_y {
            for x in world.map.min_x..=world.map.max_x {
                world.map.set_cell(Position::new(x, y), Cell::Empty);
            }
        }
        world.environment.sites.clear();
        world
            .map
            .set_cell(Position::new(0, 1), Cell::Food { amount: 3 });
        world
            .map
            .set_cell(Position::new(3, 0), Cell::Water { amount: 5 });
        world
            .map
            .set_cell(Position::new(-3, 0), Cell::Water { amount: 5 });
        world.entities[0].learner = Learner {
            enabled: true,
            context_ttl_enabled: true,
            ..Learner::default()
        };

        let result = world.resolver_order_lab_tick().unwrap().0;
        assert_eq!(
            result["common_prepared"]["candidate_overrides"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(result["references"]["c_matches_native_on"], true);
        assert_eq!(
            result["candidate_b_equals_c_pre_resolution_audit"]["learner_interruptions_equal"],
            true
        );
        assert_eq!(
            result["branches"]["C"]["pre_state"]["entities"][0]["learner"]["episode"],
            Value::Null
        );
        assert_eq!(
            result["branches"]["C"]["pre_state"]["entities"][0]["learner"]["last_episode"]["end"],
            "Interrupted"
        );
        assert_eq!(
            result["branches"]["C"]["pre_state"]["entities"][0]["learner"]["last_episode"]["reward"],
            Value::Null
        );
        assert_eq!(
            result["branches"]["C"]["post_state"]["entities"][0]["life"]["energy"],
            3
        );
        assert!(
            result["branches"]["C"]["events"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e
                    .get("EntityAte")
                    .is_some_and(|ate| ate["id"] == 1 && ate["amount"] == 3))
        );
    }

    #[test]
    fn fixed_candidate_order_keeps_baseline_survivors_and_other_order() {
        let mut baseline = ActionPlans::default();
        baseline.moves = vec![
            MovePlan {
                id: 6,
                path: vec![],
                new_pos: Position::new(1, 0),
                cost: 1,
            },
            MovePlan {
                id: 2,
                path: vec![],
                new_pos: Position::new(2, 0),
                cost: 1,
            },
        ];
        baseline.eats = vec![eat(8, 8), eat(3, 3), eat(5, 5)];
        baseline.drinks = vec![
            DrinkPlan {
                id: 9,
                path: vec![],
                new_pos: Position::new(9, 0),
                cost: 1,
                resource_pos: Position::new(9, 0),
            },
            DrinkPlan {
                id: 4,
                path: vec![],
                new_pos: Position::new(4, 0),
                cost: 1,
                resource_pos: Position::new(4, 0),
            },
        ];
        let mut candidate = baseline.clone();
        candidate.eats = vec![eat(7, 7), eat(5, 5), eat(1, 1), eat(3, 3)];
        candidate.drinks = vec![baseline.drinks[0].clone()];
        let fixed = fixed_candidate_order(&baseline, &candidate, 13);
        assert_eq!(
            fixed.eats.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![3, 5, 1, 7]
        );
        assert_eq!(
            fixed.drinks.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![9]
        );
        assert_eq!(
            fixed.moves.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![6, 2]
        );
        assert_eq!(plan_content(&fixed), plan_content(&candidate));
    }
}
