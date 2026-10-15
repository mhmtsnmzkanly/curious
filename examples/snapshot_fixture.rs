fn main() {
    let mut w = serde_json::from_str::<curious::scenario::Scenario>(include_str!(
        "../scenarios/meadow.json"
    ))
    .unwrap()
    .build(2)
    .unwrap();
    for _ in 0..500 {
        w.tick();
    }
    println!(
        "{}",
        serde_json::to_string(&curious::persistence::State::capture(&w)).unwrap()
    );
}
