//! Explicit initial-condition generator; no population feedback during a run.
//! BASE PATCHES FOOD_MULT WATER_MULT AGE_MODE OUTPUT; AGE_MODE=zero|staggered.
use curious::{
    environment::ResourceKind,
    map::movement::Position,
    scenario::{Founder, Scenario},
};
use std::{env, fs};
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PredatorProfile {
    max_age: usize,
    max_energy: usize,
    max_meal: usize,
    reproduction_interval: usize,
    reproduction_reserve_per_mille: usize,
}
fn main() {
    let args: Vec<_> = env::args().collect();
    assert!(
        (7..=8).contains(&args.len()),
        "BASE PATCHES FOOD_MULT WATER_MULT AGE_MODE OUTPUT [PREDATOR_PROFILE]"
    );
    let profile = args
        .get(7)
        .map(|p| serde_json::from_str::<PredatorProfile>(&fs::read_to_string(p).unwrap()).unwrap());
    let raw = fs::read_to_string(&args[1]).unwrap();
    let base: Scenario = serde_json::from_str(&raw).unwrap();
    let reference = serde_json::from_str::<Scenario>(&raw)
        .unwrap()
        .build(1)
        .unwrap();
    let n: isize = args[2].parse().unwrap();
    let food: usize = args[3].parse().unwrap();
    let water: usize = args[4].parse().unwrap();
    assert!((1..=8).contains(&n) && (1..=8).contains(&food) && (1..=8).contains(&water));
    assert!(["zero", "staggered", "steady"].contains(&args[5].as_str()));
    let [min_x, max_x, min_y, max_y] = base.bounds;
    let stride_x = max_x - min_x + 2;
    let stride_y = max_y - min_y + 2;
    let mut result = Scenario {
        name: format!(
            "{} {n}x{n} food={food} water={water} age={}",
            base.name, args[5]
        ),
        bounds: [
            min_x,
            max_x + stride_x * (n - 1),
            min_y,
            max_y + stride_y * (n - 1),
        ],
        founders: vec![],
        sites: vec![],
        resources: vec![],
    };
    for y in 0..n {
        for x in 0..n {
            let shift = |p: Position| Position::new(p.x + x * stride_x, p.y + y * stride_y);
            for (i, f) in base.founders.iter().enumerate() {
                let mut life = reference.entities[i].base.life().clone();
                if f.species == curious::entity::species::Species::Carnivore {
                    if let Some(p) = &profile {
                        life.max_age = p.max_age;
                        life.max_energy = p.max_energy;
                        life.energy = p.max_energy;
                        life.max_meal = p.max_meal;
                        life.reproduction_interval = p.reproduction_interval;
                        life.reproduction_reserve_per_mille = p.reproduction_reserve_per_mille;
                        life.validate().unwrap();
                    }
                }
                if args[5] == "zero" {
                    life.age = 0;
                    life.reproduction_cooldown = 0;
                }
                if args[5] != "zero" {
                    life.age =
                        ((i + (y * n + x) as usize * 23) * 17) % (life.max_age * 3 / 4).max(1);
                }
                if args[5] == "steady" {
                    let patch = (y * n + x) as usize;
                    let ident = i + patch * base.founders.len();
                    life.energy = life.max_energy * (60 + (ident * 13 + patch * 7) % 41) / 100;
                    life.water = life.max_water * (60 + (ident * 11 + patch * 17) % 41) / 100;
                    life.reproduction_cooldown = if life.is_mature() {
                        (life.reproduction_interval
                            - (life.age - life.maturity_age) % life.reproduction_interval)
                            % life.reproduction_interval
                    } else {
                        0
                    };
                }
                result.founders.push(Founder {
                    pos: shift(f.pos),
                    species: f.species.clone(),
                    genome: f.genome.clone(),
                    life: Some(life),
                });
            }
            for site in &base.sites {
                let mut s = site.clone();
                s.pos = shift(s.pos);
                s.replenish = (s.replenish
                    * match s.kind {
                        ResourceKind::Food => food,
                        ResourceKind::Water => water,
                    })
                .min(s.capacity);
                result.sites.push(s);
            }
            for (p, c) in &base.resources {
                result.resources.push((shift(*p), c.clone()));
            }
        }
    }
    // Validate the entire generated initial condition before writing anything.
    let encoded = serde_json::to_string_pretty(&result).unwrap();
    serde_json::from_str::<Scenario>(&encoded)
        .unwrap()
        .build(1)
        .unwrap();
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[6])
        .unwrap();
    writeln!(file, "{encoded}").unwrap();
}
