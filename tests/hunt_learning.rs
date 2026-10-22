use curious::{
    creatures::carnivore::CarnivoreEntity,
    entity::{
        decision::{Decision, Reason},
        hunt_learning::{
            HuntContext, HuntEnd, HuntLearner, HuntLedger, HuntMeal, HuntValue, MAX_HUNT_TICKS,
            MAX_HUNT_VALUES,
        },
        intent::Intent,
        perception::Perception,
        species::Species,
    },
    map::movement::{Direction, Steps},
    random::Random,
};
fn life() -> curious::entity::lifestate::LifeState {
    CarnivoreEntity::default().life_state
}
fn view() -> Perception {
    let mut v = Perception::empty();
    v.add_entity(
        10,
        Species::Herbivore,
        100,
        Steps(vec![Direction::Right]),
        false,
    );
    v.add_entity(
        20,
        Species::Herbivore,
        100,
        Steps(vec![Direction::Up; 2]),
        false,
    );
    v.entities[0].health = Some(20);
    v.entities[1].health = Some(40);
    v
}
fn base() -> Decision {
    Decision::new(Intent::Attack { target_id: 10 }, Reason::HuntPrey)
}
fn learner() -> HuntLearner {
    HuntLearner {
        enabled: true,
        ..Default::default()
    }
}
fn choose(l: &mut HuntLearner, seed: u64) -> Decision {
    l.choose(
        Species::Carnivore,
        &life(),
        &view(),
        base(),
        &mut Random::new(seed),
    )
}
fn fed(l: &mut HuntLearner, food: usize) {
    let id = l.episode.as_ref().unwrap().target;
    l.finish(
        1,
        &HuntLedger {
            target_killed_by: Some(1),
            attack_energy: 3,
            food: vec![HuntMeal {
                corpse_id: id,
                amount: food,
            }],
            ..Default::default()
        },
    );
}
#[test]
fn disabled_is_strict_state_and_rng_noop() {
    let mut l = HuntLearner::default();
    let before = serde_json::to_value(&l).unwrap();
    let mut rng = Random::new(77);
    let d = l.choose(Species::Carnivore, &life(), &view(), base(), &mut rng);
    l.finish(1, &HuntLedger::default());
    assert_eq!(d.intent, base().intent);
    assert_eq!(d.reason, Reason::HuntPrey);
    assert_eq!(rng.state(), 77);
    assert_eq!(serde_json::to_value(l).unwrap(), before);
}
#[test]
fn same_view_same_age_different_real_experience_changes_choice() {
    let mut good = learner();
    choose(&mut good, 50);
    fed(&mut good, 10);
    let mut bad = learner();
    choose(&mut bad, 50);
    fed(&mut bad, 0);
    assert_eq!(good.last_episode.as_ref().unwrap().attempt.target, 10);
    assert_eq!(bad.episode.as_ref().unwrap().target, 10);
    // Zero-food ledger doesn't close until the bounded timeout.
    for _ in 1..MAX_HUNT_TICKS {
        bad.finish(1, &HuntLedger::default());
    }
    choose(&mut good, 50);
    choose(&mut bad, 50);
    assert_eq!(good.episode.as_ref().unwrap().target, 10);
    assert_eq!(bad.episode.as_ref().unwrap().target, 20);
}
#[test]
fn old_preference_reverses_after_new_evidence() {
    let mut l = learner();
    choose(&mut l, 50);
    fed(&mut l, 10);
    let c = l.values[0].context;
    // One visible candidate makes the repeated deteriorating outcome causal
    // evidence for the same context, rather than hiding it behind exploration.
    let mut v = view();
    v.entities.truncate(1);
    for _ in 0..20 {
        l.choose(
            Species::Carnivore,
            &life(),
            &v,
            base(),
            &mut Random::new(50),
        );
        for _ in 0..MAX_HUNT_TICKS {
            l.finish(
                1,
                &HuntLedger {
                    movement_energy: 1,
                    ..Default::default()
                },
            );
        }
    }
    assert!(l.values.iter().find(|x| x.context == c).unwrap().value < 0);
    choose(&mut l, 50);
    assert_eq!(l.episode.as_ref().unwrap().target, 20);
}
#[test]
fn frozen_policy_observes_costs_but_never_updates_values() {
    let mut l = learner();
    choose(&mut l, 50);
    fed(&mut l, 10);
    l.update_enabled = false;
    let before = l.values.clone();
    choose(&mut l, 50);
    fed(&mut l, 1);
    assert_eq!(l.values, before);
    assert!(!l.last_episode.as_ref().unwrap().updated);
    assert!(l.last_episode.as_ref().unwrap().reward.is_some());
}
#[test]
fn target_corpse_and_own_kill_are_both_required_for_food_credit() {
    let mut l = learner();
    choose(&mut l, 50);
    l.finish(
        1,
        &HuntLedger {
            target_killed_by: Some(2),
            food: vec![HuntMeal {
                corpse_id: 10,
                amount: 30,
            }],
            ..Default::default()
        },
    );
    let result = l.last_episode.as_ref().unwrap();
    assert_eq!(result.end, HuntEnd::ForeignKill);
    assert_eq!(result.reward, None);
    assert_eq!(result.attempt.food, 0);
    assert!(l.values.is_empty());
    choose(&mut l, 50);
    l.finish(
        1,
        &HuntLedger {
            target_killed_by: Some(1),
            food: vec![HuntMeal {
                corpse_id: 999,
                amount: 30,
            }],
            ..Default::default()
        },
    );
    assert_eq!(l.episode.as_ref().unwrap().food, 0);
    let eat = Decision::new(
        Intent::Eat {
            at: Steps(vec![]),
            corpse_id: Some(10),
        },
        Reason::SeekFood,
    );
    let d = l.choose(
        Species::Carnivore,
        &life(),
        &Perception::empty(),
        eat.clone(),
        &mut Random::new(50),
    );
    assert_eq!(d.intent, eat.intent);
    assert!(l.episode.is_some());
    l.finish(
        1,
        &HuntLedger {
            movement_energy: 2,
            attack_energy: 3,
            food: vec![HuntMeal {
                corpse_id: 10,
                amount: 10,
            }],
            ..Default::default()
        },
    );
    let result = l.last_episode.as_ref().unwrap();
    assert_eq!(result.end, HuntEnd::Fed);
    assert_eq!(result.reward, Some(100 - 4 - 6 - 2));
    assert!(result.updated);
}
#[test]
fn injury_and_missing_provenance_do_not_fabricate_rewards() {
    for ledger in [
        HuntLedger {
            incoming_damage: 5,
            ..Default::default()
        },
        HuntLedger {
            ambiguous: true,
            ..Default::default()
        },
    ] {
        let mut l = learner();
        choose(&mut l, 50);
        l.finish(1, &ledger);
        fed(&mut l, 10);
        assert_eq!(l.last_episode.as_ref().unwrap().reward, None);
        assert!(l.values.is_empty());
    }
    let mut l = learner();
    choose(&mut l, 50);
    l.finish(
        1,
        &HuntLedger {
            food: vec![HuntMeal {
                corpse_id: 10,
                amount: 10,
            }],
            ..Default::default()
        },
    );
    for _ in 1..MAX_HUNT_TICKS {
        l.finish(1, &HuntLedger::default());
    }
    assert_eq!(l.last_episode.as_ref().unwrap().reward, None);
}

#[test]
fn only_attributed_prey_defense_damage_can_coexist_with_reward() {
    let mut own = learner();
    choose(&mut own, 50);
    own.finish(
        1,
        &HuntLedger {
            incoming_damage: 5,
            prey_defense_damage: 5,
            ..Default::default()
        },
    );
    fed(&mut own, 10);
    assert_eq!(
        own.last_episode.as_ref().unwrap().reward,
        Some(100 - 3 * 2 - 2)
    );

    let mut mixed = learner();
    choose(&mut mixed, 50);
    mixed.finish(
        1,
        &HuntLedger {
            incoming_damage: 7,
            prey_defense_damage: 5,
            external_damage: 2,
            ..Default::default()
        },
    );
    fed(&mut mixed, 10);
    assert_eq!(mixed.last_episode.as_ref().unwrap().reward, None);
    assert!(mixed.last_episode.as_ref().unwrap().attempt.ambiguous);

    let mut malformed = learner();
    choose(&mut malformed, 50);
    malformed.finish(
        1,
        &HuntLedger {
            incoming_damage: 5,
            prey_defense_damage: 4,
            ..Default::default()
        },
    );
    fed(&mut malformed, 10);
    assert_eq!(malformed.last_episode.as_ref().unwrap().reward, None);
}

#[test]
fn visible_target_own_energy_cutoff_scores_only_prior_audited_cost_once() {
    let mut one_target = view();
    one_target.entities.truncate(1);
    one_target.entities[0].steps = Steps(vec![Direction::Right, Direction::Right]);
    let mut l = learner();
    l.choose(
        Species::Carnivore,
        &life(),
        &one_target,
        base(),
        &mut Random::new(50),
    );
    l.finish(
        1,
        &HuntLedger {
            movement_energy: 4,
            ..Default::default()
        },
    );
    let prior = l.episode.as_ref().unwrap().clone();
    let mut low = life();
    low.energy = 3;
    let reason = curious::entity::hunting::decide(Species::Carnivore, &low, &one_target).unwrap();
    assert_eq!(reason.reason, Reason::HuntLowEnergy);
    assert_eq!(
        l.choose(
            Species::Carnivore,
            &low,
            &one_target,
            reason.clone(),
            &mut Random::new(50),
        )
        .reason,
        Reason::HuntLowEnergy
    );
    let result = l.last_episode.as_ref().unwrap();
    assert_eq!(result.end, HuntEnd::Interrupted); // V13 wire value remains stable
    assert_eq!(result.attempt, prior);
    assert_eq!(result.reward, Some(-8 - i32::from(prior.ticks)));
    assert!(result.updated);
    assert!(l.episode.is_none());
    assert_eq!(l.values[0].visits, 1);

    // External injury already makes the same apparent budget stop ambiguous.
    let mut censored = learner();
    censored.choose(
        Species::Carnivore,
        &life(),
        &one_target,
        base(),
        &mut Random::new(50),
    );
    censored.finish(
        1,
        &HuntLedger {
            movement_energy: 4,
            incoming_damage: 2,
            external_damage: 2,
            ..Default::default()
        },
    );
    censored.choose(
        Species::Carnivore,
        &low,
        &one_target,
        reason,
        &mut Random::new(50),
    );
    assert_eq!(censored.last_episode.as_ref().unwrap().reward, None);

    let mut costly_view = one_target.clone();
    costly_view.entities[0].health = Some(60);
    let mut costly = learner();
    costly.choose(
        Species::Carnivore,
        &life(),
        &costly_view,
        base(),
        &mut Random::new(50),
    );
    costly.finish(
        1,
        &HuntLedger {
            movement_energy: 4,
            ..Default::default()
        },
    );
    let mut limited = life();
    limited.energy = 10;
    let reason =
        curious::entity::hunting::decide(Species::Carnivore, &limited, &costly_view).unwrap();
    assert_eq!(reason.reason, Reason::HuntTooCostly);
    costly.choose(
        Species::Carnivore,
        &limited,
        &costly_view,
        reason,
        &mut Random::new(50),
    );
    assert!(costly.last_episode.as_ref().unwrap().reward.is_some());
}

#[test]
fn old_ambiguous_pending_episode_stays_censored_after_checkpoint_roundtrip() {
    let mut l = learner();
    choose(&mut l, 50);
    l.episode.as_mut().unwrap().ambiguous = true;
    l.finish(
        1,
        &HuntLedger {
            movement_energy: 4,
            prey_defense_damage: 6,
            incoming_damage: 6,
            ..Default::default()
        },
    );
    let mut restored: HuntLearner =
        serde_json::from_value(serde_json::to_value(&l).unwrap()).unwrap();
    fed(&mut restored, 10);
    assert_eq!(restored.last_episode.as_ref().unwrap().reward, None);
    assert!(restored.last_episode.as_ref().unwrap().attempt.ambiguous);
}
#[test]
fn priority_physical_filters_and_unknown_health_are_respected() {
    let mut l = learner();
    let mut rng = Random::new(50);
    let urgent = Decision::new(Intent::Idle { duration: 1 }, Reason::RecoverStamina);
    assert_eq!(
        l.choose(Species::Carnivore, &life(), &view(), urgent, &mut rng)
            .reason,
        Reason::RecoverStamina
    );
    assert!(l.episode.is_none());
    assert_eq!(rng.state(), 50);
    let mut exhausted = life();
    exhausted.stamina = 0;
    assert_eq!(
        l.choose(Species::Carnivore, &exhausted, &view(), base(), &mut rng)
            .reason,
        Reason::HuntPrey
    );
    assert!(l.episode.is_none());
    let mut v = view();
    v.entities[0].health = None;
    assert_eq!(
        HuntContext::from_seen(&life(), &v.entities[0], 20).health,
        0
    );
    let mut cheap = life();
    cheap.energy = 3;
    l.choose(Species::Carnivore, &cheap, &v, base(), &mut rng);
    assert_eq!(l.episode.as_ref().unwrap().target, 10); // only adjacent physically affordable
}
#[test]
fn checkpoint_resumes_pending_episode_rng_and_knowledge_exactly() {
    let mut a = learner();
    let mut ra = Random::new(50);
    a.choose(Species::Carnivore, &life(), &view(), base(), &mut ra);
    a.finish(
        1,
        &HuntLedger {
            movement_energy: 2,
            ..Default::default()
        },
    );
    let mut b: HuntLearner = serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap();
    let mut rb = Random::new(ra.state());
    fed(&mut a, 10);
    fed(&mut b, 10);
    let da = a.choose(Species::Carnivore, &life(), &view(), base(), &mut ra);
    let db = b.choose(Species::Carnivore, &life(), &view(), base(), &mut rb);
    assert_eq!(da.intent, db.intent);
    assert_eq!(ra.state(), rb.state());
    assert_eq!(a, b);
    // Newborn constructor has no inheritance path.
    assert!(HuntLearner::default().values.is_empty());
}
#[test]
fn context_space_is_finite_and_corrupt_serialized_state_is_rejected() {
    let mut l = learner();
    for energy in 0..3 {
        for stamina in 0..3 {
            for distance in 0..3 {
                for health in 0..4 {
                    l.values.push(HuntValue {
                        context: HuntContext {
                            energy,
                            stamina,
                            distance,
                            health,
                        },
                        value: 0,
                        visits: 1,
                    });
                }
            }
        }
    }
    assert_eq!(l.values.len(), MAX_HUNT_VALUES);
    l.validate().unwrap();
    l.values.push(l.values[0].clone());
    assert!(l.validate().is_err());
    l.values.pop();
    l.values[0].context.health = 4;
    assert!(l.validate().is_err());
}
#[test]
fn episode_timeout_is_twenty_ticks_and_lost_targets_are_uncertain() {
    let mut l = learner();
    choose(&mut l, 50);
    for _ in 0..MAX_HUNT_TICKS - 1 {
        l.finish(1, &HuntLedger::default());
        assert!(l.episode.is_some());
    }
    l.finish(1, &HuntLedger::default());
    assert!(l.episode.is_none());
    assert_eq!(l.last_episode.as_ref().unwrap().end, HuntEnd::TimedOut);
    assert_eq!(l.last_episode.as_ref().unwrap().attempt.ticks, 20);
    choose(&mut l, 50);
    l.choose(
        Species::Carnivore,
        &life(),
        &Perception::empty(),
        base(),
        &mut Random::new(50),
    );
    assert_eq!(l.last_episode.as_ref().unwrap().reward, None);
    assert_eq!(l.last_episode.as_ref().unwrap().end, HuntEnd::TargetLost);
}

#[test]
fn v2_equal_values_preserve_rule_cost_ordering_outside_exploration() {
    for (near_health, near_power, far_health, far_steps) in [
        (Some(100), 100, Some(20), 3),
        (None, 200, Some(20), 2),
        (Some(20), 100, Some(100), 3),
        (Some(20), 100, Some(20), 1),
    ] {
        let mut seen = view();
        seen.entities[0].health = near_health;
        seen.entities[0].power = near_power;
        seen.entities[1].health = far_health;
        seen.entities[1].steps = Steps(vec![Direction::Up; far_steps]);
        let body = life();
        let rule = curious::entity::hunting::decide(Species::Carnivore, &body, &seen).unwrap();
        assert_eq!(rule.reason, Reason::HuntPrey);
        for trained_equal in [false, true] {
            let mut l = learner();
            if trained_equal {
                for prey in &seen.entities {
                    let context = HuntContext::from_seen(&body, prey, 20);
                    if !l.values.iter().any(|v| v.context == context) {
                        l.values.push(HuntValue {
                            context,
                            value: 17,
                            visits: 3,
                        });
                    }
                }
            }
            // RNG's first value is 50, explicitly outside the 0..9 epsilon arm.
            let selected = l.choose(
                Species::Carnivore,
                &body,
                &seen,
                rule.clone(),
                &mut Random::new(50),
            );
            assert_eq!(
                selected.intent, rule.intent,
                "equal Q must not replace rule cost ranking"
            );
        }
    }
}
#[test]
fn v2_learned_value_still_overrides_the_rule_cost_tiebreaker() {
    let body = life();
    let seen = view();
    let rule = curious::entity::hunting::decide(Species::Carnivore, &body, &seen).unwrap();
    assert_eq!(rule.intent, Intent::Attack { target_id: 10 });
    let mut l = learner();
    l.values.push(HuntValue {
        context: HuntContext::from_seen(&body, &seen.entities[1], 20),
        value: 50,
        visits: 3,
    });
    l.choose(Species::Carnivore, &body, &seen, rule, &mut Random::new(50));
    assert_eq!(l.episode.as_ref().unwrap().target, 20);
}

#[test]
fn v3_reranks_visible_targets_without_transferring_previous_costs() {
    let body = life();
    let mut seen = view();
    let mut l = learner();
    l.choose(
        Species::Carnivore,
        &body,
        &seen,
        base(),
        &mut Random::new(50),
    );
    l.finish(
        1,
        &HuntLedger {
            movement_energy: 7,
            attack_energy: 3,
            ..Default::default()
        },
    );
    let old = l.episode.as_ref().unwrap().clone();
    assert_eq!(old.target, 10);
    // Both still visible; the previously more expensive prey is now wounded.
    seen.entities[0].health = Some(100);
    seen.entities[1].health = Some(1);
    let rule = curious::entity::hunting::decide(Species::Carnivore, &body, &seen).unwrap();
    assert!(matches!(rule.intent, Intent::Move { .. }));
    let chosen = l.choose(
        Species::Carnivore,
        &body,
        &seen,
        rule.clone(),
        &mut Random::new(50),
    );
    assert_eq!(chosen.intent, rule.intent);
    let closed = l.last_episode.as_ref().unwrap();
    assert_eq!(closed.end, HuntEnd::Interrupted);
    assert_eq!(closed.reward, None);
    assert_eq!(closed.attempt, old);
    assert!(l.values.is_empty());
    let new = l.episode.as_ref().unwrap();
    assert_eq!(new.target, 20);
    assert_eq!(new.ticks, 0);
    assert_eq!(new.movement_energy, 0);
    assert_eq!(new.attack_energy, 0);
}
#[test]
fn v3_same_target_keeps_initial_context_and_complete_accounting() {
    let body = life();
    let seen = view();
    let mut l = learner();
    l.choose(
        Species::Carnivore,
        &body,
        &seen,
        base(),
        &mut Random::new(50),
    );
    l.finish(
        1,
        &HuntLedger {
            movement_energy: 2,
            attack_energy: 3,
            ..Default::default()
        },
    );
    let previous = l.episode.as_ref().unwrap().clone();
    let mut changed_body = body;
    changed_body.energy = 35; // different context band
    let rule = curious::entity::hunting::decide(Species::Carnivore, &changed_body, &seen).unwrap();
    let chosen = l.choose(
        Species::Carnivore,
        &changed_body,
        &seen,
        rule.clone(),
        &mut Random::new(50),
    );
    assert_eq!(chosen.intent, rule.intent);
    assert_eq!(l.episode.as_ref().unwrap(), &previous);
    assert!(l.last_episode.is_none());
    l.finish(
        1,
        &HuntLedger {
            movement_energy: 4,
            attack_energy: 3,
            ..Default::default()
        },
    );
    let continued = l.episode.as_ref().unwrap();
    assert_eq!(continued.ticks, 2);
    assert_eq!(continued.movement_energy, 6);
    assert_eq!(continued.attack_energy, 6);
    assert_eq!(continued.context, previous.context);
}
