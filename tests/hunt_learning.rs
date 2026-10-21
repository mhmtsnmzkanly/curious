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
