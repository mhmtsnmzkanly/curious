use curious::{
    entity::{decision::Reason, intent::Intent},
    persistence::{State, Store},
    scenario::Scenario,
};
fn meadow() -> curious::world::World {
    serde_json::from_str::<Scenario>(include_str!("../scenarios/meadow.json"))
        .unwrap()
        .build(2)
        .unwrap()
}
#[test]
fn observation_does_not_change_decisions_or_random_stream() {
    let mut compact = meadow();
    let mut full = meadow();
    full.observation.full_all = true;
    for _ in 0..100 {
        let a = compact.tick();
        let b = full.tick();
        assert_eq!(
            serde_json::to_value(a).unwrap(),
            serde_json::to_value(b).unwrap()
        );
        let a = State::capture(&compact);
        let mut b = State::capture(&full);
        b.observation = a.observation.clone();
        assert_eq!(
            serde_json::to_value(a).unwrap(),
            serde_json::to_value(b).unwrap()
        );
        assert!(
            compact
                .last_decisions
                .iter()
                .all(|d| d.perception.is_none())
        );
        assert!(full.last_decisions.iter().all(|d| d.perception.is_some()));
    }
}
#[test]
fn external_commands_have_honest_reasons_and_actual_outcomes() {
    let mut w = meadow();
    w.push_command(1, Intent::Idle { duration: 1 });
    w.tick();
    let trace = w.last_decisions.iter().find(|d| d.id == 1).unwrap();
    assert_eq!(trace.decision.reason, Reason::ExternalCommand);
    assert_eq!(trace.pos, trace.outcome.as_ref().unwrap().pos);
    assert!(trace.outcome.as_ref().unwrap().events.is_empty());
}
#[test]
fn traces_commit_with_the_tick_and_schema_is_versioned() {
    let dir = tempfile::tempdir().unwrap();
    let mut w = meadow();
    let events = w.tick();
    let mut store = Store::open(dir.path(), 1024 * 1024).unwrap();
    store.record(&w, &events, "tick", 0, None).unwrap();
    let q = store
        .query(0, "SELECT count(*) FROM decision_traces")
        .unwrap();
    assert_eq!(q["rows"][0][0], 8);
    let q = store
        .query(
            0,
            "SELECT json_extract(trace_json,'$.outcome.phase') FROM decision_traces LIMIT 1",
        )
        .unwrap();
    assert_eq!(q["rows"][0][0], "Active");
    drop(store);
    let db = rusqlite::Connection::open(dir.path().join("history-000000.sqlite")).unwrap();
    db.pragma_update(None, "user_version", 99).unwrap();
    drop(db);
    assert!(Store::open(dir.path(), 1024 * 1024).is_err());
}
