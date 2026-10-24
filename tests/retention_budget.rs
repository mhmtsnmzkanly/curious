use curious::{
    persistence::{State, Store},
    world::World,
};

fn archive_bytes(store: &Store) -> u64 {
    store
        .inventory()
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|shard| shard["compressed"] == true)
        .map(|shard| shard["bytes"].as_u64().unwrap())
        .sum()
}

fn active_page_bytes(dir: &std::path::Path, shard: usize) -> u64 {
    let conn = rusqlite::Connection::open(dir.join(format!("history-{shard:06}.sqlite"))).unwrap();
    let pages: i64 = conn
        .query_row("PRAGMA page_count", [], |row| row.get(0))
        .unwrap();
    let size: i64 = conn
        .query_row("PRAGMA page_size", [], |row| row.get(0))
        .unwrap();
    (pages * size) as u64
}

#[test]
fn external_retention_recounts_before_budget_stop_and_resume_uses_real_files() {
    let dir = tempfile::tempdir().unwrap();
    let mut world = World::with_seed(-1, 1, -1, 1, Vec::new(), 731);
    world.logger = curious::logger::Logger::disabled();
    let mut store = Store::open(dir.path(), 1).unwrap();

    // Force each record into a new active shard, leaving closed shards behind.
    for _ in 0..24 {
        world.tick();
        store.record(&world, &[], "tick", 0, None).unwrap();
    }
    let before = store
        .inventory()
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|shard| shard["compressed"] == true)
        .map(|shard| {
            (
                shard["shard"].as_u64().unwrap() as usize,
                shard["bytes"].as_u64().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    assert!(
        before.len() >= 8,
        "expected multiple sealed shards: {before:?}"
    );
    let newest = before.iter().map(|(n, _)| *n).max().unwrap();
    let mut removed_bytes = 0;
    for (number, bytes) in &before {
        if *number != newest {
            let path = dir.path().join(format!("history-{number:06}.sqlite.zst"));
            if path.exists() {
                std::fs::remove_file(path).unwrap();
                removed_bytes += *bytes;
            } else {
                let path = dir.path().join(format!("history-{number:06}.sqlite.gz"));
                if path.exists() {
                    std::fs::remove_file(path).unwrap();
                    removed_bytes += *bytes;
                }
            }
        }
    }
    assert!(removed_bytes > 1);

    // This limit is just above the measured remaining archives + active pages,
    // but below the stale cached projection from the deleted archives.
    let remaining_archives = archive_bytes(&store);
    let active_bytes = active_page_bytes(dir.path(), store.shard);
    let corrected_limit = remaining_archives + active_bytes + 1;
    assert!(
        remaining_archives + removed_bytes + active_bytes >= corrected_limit,
        "the stale cache must have triggered the recount"
    );
    store.max_history_bytes = corrected_limit;
    world.tick();
    store
        .record(&world, &[], "after-retention", 0, None)
        .unwrap();

    // The guard still stops when the actual current total is over budget.
    let actual_archives = archive_bytes(&store);
    let actual_active = active_page_bytes(dir.path(), store.shard);
    store.max_history_bytes = actual_archives + actual_active - 1;
    assert!(store.record(&world, &[], "over-budget", 0, None).is_err());

    // The last accepted checkpoint retains state and RNG and can resume from the
    // real on-disk inventory after reopening.
    let saved = store.latest().unwrap().unwrap();
    let restored = saved.clone().restore().unwrap();
    assert_eq!(
        serde_json::to_value(State::capture(&restored)).unwrap(),
        serde_json::to_value(&saved).unwrap()
    );
    let expected_rng = saved.rng;
    drop(store);
    let reopened = Store::open(dir.path(), 1).unwrap();
    assert_eq!(reopened.latest().unwrap().unwrap().rng, expected_rng);
    assert_eq!(
        reopened.estimated_bytes().unwrap(),
        archive_bytes(&reopened) + active_page_bytes(dir.path(), reopened.shard)
    );
}

#[cfg(unix)]
#[test]
fn dangling_closed_archive_fails_open_instead_of_counting_as_zero_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let broken_archive = dir.path().join("history-000000.sqlite.zst");
    std::os::unix::fs::symlink("missing-archive-target", &broken_archive).unwrap();

    assert!(Store::open(dir.path(), 1024 * 1024).is_err());
}
