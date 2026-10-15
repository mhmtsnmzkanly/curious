//! Engine-only throughput: no SQLite or HTTP. Run each case in its own process.
use curious::{
    creatures::herbivore::HerbivoreEntity,
    entity::{controller::EntityController, phase::EntityPhase},
    map::{Map, cell::Cell, movement::Position},
    persistence::State,
    world::{EntitySlot, World},
};
use std::{env, time::Instant};
fn main() {
    let args: Vec<_> = env::args().collect();
    let n: usize = args[1].parse().unwrap();
    let side: isize = args[2].parse().unwrap();
    let ticks: usize = args[3].parse().unwrap();
    let forage = args.get(4).is_some_and(|s| s == "forage");
    let entities = (0..n)
        .map(|i| {
            let mut s = EntitySlot::new(
                i + 1,
                Position::new(i as isize % side, i as isize / side),
                EntityPhase::Active,
                EntityController::AI,
                Box::new(HerbivoreEntity::default()),
            );
            if forage {
                s.base.life_mut().energy = 20;
            }
            s
        })
        .collect();
    let mut w = World::with_seed(0, side - 1, 0, side - 1, entities, 42);
    w.map = Map::new(0, side - 1, 0, side - 1);
    w.environment = Default::default();
    w.logger = curious::logger::Logger::disabled();
    if forage {
        for x in (0..side).step_by(4) {
            for y in (0..side).step_by(4) {
                w.map
                    .set_cell(Position::new(x, y), Cell::Food { amount: 100 });
            }
        }
    }
    let mut timings = Vec::new();
    for _ in 0..ticks {
        let now = Instant::now();
        w.tick();
        timings.push(now.elapsed().as_micros());
    }
    timings.sort();
    let status = std::fs::read_to_string("/proc/self/status").unwrap();
    let rss = status
        .lines()
        .find(|s| s.starts_with("VmHWM:"))
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap();
    println!(
        "{}",
        serde_json::json!({"population":n,"side":side,"ticks":ticks,"forage":forage,"p50_us":timings[ticks/2],"p95_us":timings[ticks*95/100],"rss_high_water_kib":rss.parse::<usize>().unwrap(),"final_entities":w.entities.len(),"checkpoint_bytes":serde_json::to_vec(&State::capture(&w)).unwrap().len()})
    );
}
