use curious::{archive, persistence::Store, world::World};
use std::io::Read;

#[test]
fn a_sealed_last_shard_can_be_queried_and_resumed() {
    let dir = tempfile::tempdir().unwrap();
    let mut world = World::with_seed(-1, 1, -1, 1, Vec::new(), 7);
    world.logger = curious::logger::Logger::disabled();
    let mut store = Store::open(dir.path(), 1024 * 1024).unwrap();
    store.record(&world, &[], "initial", 0, None).unwrap();
    drop(store);
    let raw = dir.path().join("history-000000.sqlite");
    let original = std::fs::read(&raw).unwrap();
    let compressed = archive::seal(&raw).unwrap();
    assert!(!raw.exists());
    let mut decoded = Vec::new();
    zstd::stream::read::Decoder::new(std::fs::File::open(&compressed).unwrap())
        .unwrap()
        .read_to_end(&mut decoded)
        .unwrap();
    assert_eq!(original, decoded);
    let mut reopened = Store::open(dir.path(), 1024 * 1024).unwrap();
    assert_eq!(reopened.shard, 1);
    assert_eq!(reopened.latest().unwrap().unwrap().tick, 0);
    let history = reopened.query(0, "SELECT count(*) FROM records").unwrap();
    assert_eq!(history["rows"][0][0], 1);
    world.tick();
    reopened.record(&world, &[], "tick", 0, None).unwrap();
    assert_eq!(reopened.latest().unwrap().unwrap().tick, 1);
}

#[test]
fn damaged_archives_fail_instead_of_returning_empty_history() {
    let dir = tempfile::tempdir().unwrap();
    let raw = dir.path().join("history-000000.sqlite");
    let c = rusqlite::Connection::open(&raw).unwrap();
    c.execute_batch("CREATE TABLE test(id INTEGER); INSERT INTO test VALUES(42)")
        .unwrap();
    drop(c);
    let path = archive::seal(&raw).unwrap();
    let mut bytes = std::fs::read(&path).unwrap();
    let end = bytes.len() - 5;
    bytes[end] ^= 0xff;
    std::fs::write(&path, bytes).unwrap();
    assert!(archive::Reader::open(&path).is_err());
}

#[test]
fn interrupted_archive_scratch_is_cleaned_without_touching_history() {
    let dir = tempfile::tempdir().unwrap();
    let temporary = dir.path().join(".curious-archive-interrupted");
    std::fs::write(&temporary, b"partial").unwrap();
    let keep = dir.path().join("notes.txt");
    std::fs::write(&keep, b"keep").unwrap();
    let _store = Store::open(dir.path(), 1024 * 1024).unwrap();
    assert!(!temporary.exists());
    assert!(keep.exists());
}

#[test]
fn legacy_gzip_history_remains_readable() {
    use std::io::Write;
    let dir = tempfile::tempdir().unwrap();
    let raw = dir.path().join("history-000000.sqlite");
    let connection = rusqlite::Connection::open(&raw).unwrap();
    connection
        .execute_batch("CREATE TABLE test(id INTEGER); INSERT INTO test VALUES(42)")
        .unwrap();
    drop(connection);
    let gzip = raw.with_extension("sqlite.gz");
    let mut encoder = flate2::write::GzEncoder::new(
        std::fs::File::create(&gzip).unwrap(),
        flate2::Compression::fast(),
    );
    encoder.write_all(&std::fs::read(&raw).unwrap()).unwrap();
    encoder.finish().unwrap();
    std::fs::remove_file(raw).unwrap();
    let reader = archive::Reader::open(&gzip).unwrap();
    let value: i64 = reader
        .query_row("SELECT id FROM test", [], |row| row.get(0))
        .unwrap();
    assert_eq!(value, 42);
}
