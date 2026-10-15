use curious::{
    control::Runtime,
    creatures::herbivore::HerbivoreEntity,
    entity::{controller::EntityController, phase::EntityPhase},
    map::movement::Position,
    persistence::{State, Store},
    world::{EntitySlot, World},
};
use serde_json::json;

#[test]
fn durable_controls_rotation_and_resume() {
    let dir = std::env::temp_dir().join(format!(
        "curious-test-{}-{}",
        std::process::id(),
        curious::persistence::now_ms()
    ));
    curious::set_global_seed(42);
    let mut world = World::new(
        -8,
        8,
        -8,
        8,
        vec![EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        )],
    );
    world.logger = curious::logger::Logger::disabled();
    // Deliberately tiny threshold exercises multiple history shards.
    let mut store = Store::open(&dir, 65536).unwrap();
    store.record(&world, &[], "initial", 0, None).unwrap();
    let mut rt = Runtime {
        world,
        store,
        paused: true,
        tps: 3.0,
        shutdown: false,
        error: None,
    };
    rt.apply(json!({"action":"set_cell","pos":{"x":0,"y":0},"cell":{"Water":{"amount":100}}}))
        .unwrap();
    rt.apply(json!({"action":"intent","id":1,"intent":{"Drink":{"at":[]}}}))
        .unwrap();
    assert_eq!(rt.store.latest().unwrap().unwrap().commands.len(), 1);
    rt.apply(json!({"action":"step","ticks":1})).unwrap();
    let q = rt
        .store
        .query(
            rt.store.shard,
            "SELECT kind FROM events WHERE kind='EntityDrank'",
        )
        .unwrap();
    assert_eq!(q["rows"].as_array().unwrap().len(), 1);
    rt.apply(json!({"action":"speed","ticks_per_second":20}))
        .unwrap();
    assert!(
        rt.apply(json!({"action":"speed","ticks_per_second":0}))
            .is_err()
    );
    let before = serde_json::to_value(State::capture(&rt.world)).unwrap();
    assert!(
        rt.apply(json!({"action":"spawn","species":"Herbivore","pos":{"x":999,"y":0}}))
            .is_err()
    );
    assert_eq!(
        before,
        serde_json::to_value(State::capture(&rt.world)).unwrap()
    );
    // A queued input survives a checkpoint and controls the resumed tick.
    rt.apply(json!({"action":"intent","id":1,"intent":{"Idle":{"duration":1}}}))
        .unwrap();
    let saved = rt.store.latest().unwrap().unwrap();
    let mut restored = saved.clone().restore().unwrap();
    assert_eq!(
        serde_json::to_value(State::capture(&restored)).unwrap(),
        serde_json::to_value(&saved).unwrap()
    );
    // Replay several decisions and events, not just a static snapshot.
    let mut expected = Vec::new();
    for _ in 0..20 {
        let events = rt.world.tick();
        expected.push((
            serde_json::to_value(State::capture(&rt.world)).unwrap(),
            serde_json::to_value(events).unwrap(),
        ));
    }
    curious::set_global_seed(saved.rng);
    for (state, events) in expected {
        let replay = restored.tick();
        assert_eq!(events, serde_json::to_value(replay).unwrap());
        assert_eq!(
            state,
            serde_json::to_value(State::capture(&restored)).unwrap()
        );
    }
    for _ in 0..40 {
        rt.tick().unwrap();
    }
    assert!(rt.store.shard > 0);
    assert!(rt.store.inventory().unwrap().as_array().unwrap().iter().any(|s|s["compressed"]==true));
    assert!(rt.store.inventory().unwrap().as_array().unwrap().len() > 1);
    assert!(rt.store.query(0, "DELETE FROM records").is_err());
    assert!(
        rt.store
            .query(0, "WITH x AS (SELECT 1) DELETE FROM records")
            .is_err()
    );
    let old = rt.store.query(0, "SELECT COUNT(*) FROM records").unwrap();
    assert!(old["rows"][0][0].as_i64().unwrap() > 0);
    assert_eq!(
        rt.store.latest().unwrap().unwrap().tick,
        rt.world.tick_counter
    );
    let final_tick = rt.world.tick_counter;
    // Durable failures stop advancement rather than pretending writes succeeded.
    rt.store.max_history_bytes = 1;
    assert!(rt.tick().is_err());
    assert!(rt.paused && rt.error.is_some());
    assert!(rt.apply(json!({"action":"resume"})).is_err());
    drop(rt);
    let reopened = Store::open(&dir, 65536).unwrap();
    assert_eq!(reopened.latest().unwrap().unwrap().tick, final_tick);
    drop(reopened);
    std::fs::remove_dir_all(dir).unwrap();
}
